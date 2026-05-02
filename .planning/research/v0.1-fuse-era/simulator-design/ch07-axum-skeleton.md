# Concrete axum skeleton

← [back to index](./index.md)

Below is the spine. Implementer should expand handler bodies; types and wiring are the load-bearing parts.

### 7.1 `Cargo.toml` (relevant excerpts)

```toml
[dependencies]
axum = { version = "0.7", features = ["macros", "tokio", "json"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.5", features = ["trace", "cors"] }
tower-governor = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled", "chrono", "serde_json"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
rand = "0.8"
rand_chacha = "0.3"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4", features = ["derive"] }
sha2 = "0.10"

[dev-dependencies]
proptest = "1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

`reqwest` with `rustls-tls` (no openssl-sys) is required by `PROJECT.md` §Dependencies.

### 7.2 `main.rs`

```rust
use clap::Parser;
use reposix_sim::{build_router, AppState, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let state = AppState::new(&cli)?;

    let app = build_router(state.clone());
    let addr = format!("{}:{}", cli.bind, cli.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "reposix-sim listening");
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await?;
    Ok(())
}
```

`into_make_service_with_connect_info` is required by tower-governor's IP-based extractors even though we use a custom one — keeping the connect info available means fallback to IP works for unauth endpoints.

### 7.3 `state.rs`

```rust
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, RwLock};
use rusqlite::Connection;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub chaos: Arc<RwLock<crate::middleware::chaos::ChaosConfig>>,
    pub config: Arc<Config>,
}

pub struct Config {
    pub rate_limit_per_hour: u32,
    pub burst_size: u32,
    pub bind: String,
    pub port: u16,
}

impl AppState {
    pub fn new(cli: &Cli) -> anyhow::Result<Self> {
        let conn = crate::db::open_db(&cli.db)?;
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            chaos: Arc::new(RwLock::new(Default::default())),
            config: Arc::new(Config {
                rate_limit_per_hour: cli.rate_limit,
                burst_size: cli.burst,
                bind: cli.bind.clone(),
                port: cli.port,
            }),
        })
    }
}
```

### 7.4 `lib.rs` — the router wiring

```rust
use axum::{Router, routing::{get, post, patch, delete}};
use tower_governor::GovernorLayer;
use tower::ServiceBuilder;

pub fn build_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/projects", get(routes::projects::list))
        .route("/projects/:slug", get(routes::projects::get_one))
        .route("/projects/:slug/issues",
               get(routes::issues::list).post(routes::issues::create))
        .route("/projects/:slug/issues/:number",
               get(routes::issues::get_one)
                   .patch(routes::issues::patch)
                   .delete(routes::issues::delete))
        .route("/projects/:slug/issues/:number/transitions",
               get(routes::transitions::list).post(routes::transitions::apply))
        .route("/projects/:slug/permissions", get(routes::perms::effective))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::auth::auth_layer))
        .layer(GovernorLayer::new(governor_config(&state)))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::chaos::chaos_layer));

    let admin = Router::new()
        .route("/_audit", get(routes::dashboard::audit_json))
        .route("/_chaos", post(routes::dashboard::set_chaos))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::auth::admin_only));

    Router::new()
        .route("/", get(routes::dashboard::index_html))
        .route("/_health", get(|| async { axum::Json(serde_json::json!({"ok": true})) }))
        .merge(api)
        .merge(admin)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(), middleware::audit::audit_layer))
        .with_state(state)
}
```

Layer ordering recap (axum applies outer-first on requests, inner-first on responses): audit > chaos > governor > auth > handler. Audit must wrap everything so even rejected requests are logged; chaos before governor so injected delays don't burn tokens; governor before auth so unauthenticated floods are throttled; auth last so handlers always have `AgentContext`.

### 7.5 A handler in full — `issues::patch`

```rust
pub async fn patch(
    State(state): State<AppState>,
    Path((slug, number)): Path<(String, i64)>,
    Extension(ctx): Extension<AgentContext>,
    headers: HeaderMap,
    Json(payload): Json<PatchIssueRequest>,
) -> Result<Response, ApiError> {
    ctx.require("issues.update")?;
    if payload.has_state_field() {
        return Err(ApiError::bad_request(
            "use POST /transitions to change state"));
    }

    let if_match = headers.get(IF_MATCH)
        .ok_or(ApiError::precondition_required("If-Match required for PATCH"))?
        .to_str().map_err(|_| ApiError::bad_request("If-Match not utf8"))?
        .to_owned();

    let mut conn = state.db.lock().await;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let current = db::issues::fetch(&tx, &slug, number)?
        .ok_or(ApiError::not_found("issue"))?;
    let actual_etag = current.etag();
    if if_match != actual_etag {
        return Err(ApiError::conflict(json!({
            "error": "stale_etag",
            "expected": if_match,
            "actual": actual_etag,
        })));
    }
    let updated = db::issues::apply_patch(&tx, &current, &payload, &ctx.id)?;
    tx.commit()?;

    let mut resp = Json(&updated).into_response();
    resp.headers_mut().insert(ETAG, updated.etag().parse().unwrap());
    Ok(resp)
}
```

`ApiError` is a `thiserror`-derived enum that implements `IntoResponse`, mapping each variant to the JSON envelope `{ "error": "...", ... }`. Centralizing it keeps every handler under 30 lines.
