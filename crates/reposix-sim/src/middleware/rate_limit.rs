//! Per-agent token-bucket rate limit.
//!
//! Uses `governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>` kept
//! in a `DashMap<String, Arc<Limiter>>` keyed by `X-Reposix-Agent`. Each
//! limiter runs a per-second quota of `rps` with burst == `rps`.
//!
//! Denial path: 429 with body `{"error":"rate_limited","retry_after_secs":1}`
//! and header `Retry-After: 1`. The audit middleware sits OUTSIDE this
//! layer, so 429 responses are audited before returning to the client
//! (that's the `rate_limited_request_is_audited` invariant in the
//! integration test).

use std::{num::NonZeroU32, sync::Arc};

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, Response, StatusCode},
    middleware::{from_fn, Next},
};
use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use serde_json::json;

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// Attach the rate-limit middleware to `router`. `rps` is the per-agent
/// sustained rate (burst == rps).
///
/// A value of 0 is clamped to 1 to keep `NonZeroU32` happy; callers should
/// never pass 0, but the clamp keeps the service alive if they do.
///
/// # Panics
/// Only if `NonZeroU32::new(rps.max(1))` somehow returns `None`, which
/// cannot happen given `.max(1)` — `.expect` acts as a compile-time sanity
/// check rather than a runtime branch.
pub fn attach<S>(router: axum::Router<S>, rps: u32) -> axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let quota = Quota::per_second(NonZeroU32::new(rps.max(1)).expect("rps.max(1) >= 1"));
    let map: Arc<DashMap<String, Arc<Limiter>>> = Arc::new(DashMap::new());

    router.layer(from_fn(move |req: Request, next: Next| {
        let map = Arc::clone(&map);
        async move {
            let agent_id = req
                .headers()
                .get("X-Reposix-Agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("anonymous")
                .to_owned();

            let limiter = {
                if let Some(existing) = map.get(&agent_id) {
                    Arc::clone(existing.value())
                } else {
                    let new_limiter = Arc::new(RateLimiter::direct(quota));
                    map.insert(agent_id.clone(), Arc::clone(&new_limiter));
                    new_limiter
                }
            };

            match limiter.check() {
                Ok(()) => next.run(req).await,
                Err(_) => rate_limited_response(),
            }
        }
    }))
}

fn rate_limited_response() -> Response<Body> {
    let body = json!({
        "error": "rate_limited",
        "retry_after_secs": 1,
    })
    .to_string();
    let mut resp = Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("static response");
    resp.headers_mut()
        .insert("Retry-After", HeaderValue::from_static("1"));
    resp
}

#[cfg(test)]
mod tests {
    use super::attach;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn always_ok() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    #[tokio::test]
    async fn rate_limit_rps_1_denies_second_call() {
        // Rps=1 → burst = 1. First request passes, second (same agent) 429s.
        let app: Router = attach(Router::new().route("/z", get(always_ok)), 1);

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/z")
                    .header("X-Reposix-Agent", "burst")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), 204);

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/z")
                    .header("X-Reposix-Agent", "burst")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), 429);
        assert_eq!(
            second
                .headers()
                .get("Retry-After")
                .unwrap()
                .to_str()
                .unwrap(),
            "1"
        );
    }

    #[tokio::test]
    async fn rate_limit_is_per_agent() {
        // Rps=1. First call from agent "a" passes; second from "a" 429s;
        // third from "b" passes because "b" has its own bucket.
        let app: Router = attach(Router::new().route("/z", get(always_ok)), 1);
        let make = |agent: &'static str| {
            Request::builder()
                .uri("/z")
                .header("X-Reposix-Agent", agent)
                .body(Body::empty())
                .unwrap()
        };

        assert_eq!(app.clone().oneshot(make("a")).await.unwrap().status(), 204);
        assert_eq!(app.clone().oneshot(make("a")).await.unwrap().status(), 429);
        assert_eq!(app.oneshot(make("b")).await.unwrap().status(), 204);
    }
}
