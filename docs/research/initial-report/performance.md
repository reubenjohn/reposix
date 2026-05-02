# Performance Dynamics: Latency, Throughput, and Rate Limiting

> **Document status — initial design research (pre-v0.1, 2026-04-13).** Chapter of [Initial Report](../initial-report.md). Features discussed here are prospective designs from the v0.1-era research, not a description of the current implementation.

While FUSE provides exceptional context efficiency, the fundamental impedance mismatch between rapid local system calls and strictly rate-limited network APIs necessitates rigorous performance engineering.

## The Token Economics of Filesystem Interaction

The primary advantage of the FUSE architecture is its reduction of latency during the LLM inference phase. Generating tokens via a large language model is computationally expensive; minimizing the input context size directly accelerates the time-to-first-token and overall throughput. In benchmark studies comparing direct tool definitions to filesystem exploration, allowing an agent to discover tools and data sequentially via a filesystem reduced the required token volume from roughly 150,000 tokens to merely 2,000 tokens—yielding a 98.7% improvement in efficiency and cost. By representing an entire Jira backlog as an indexed directory structure, the agent only reads the files necessary for its immediate task, rather than parsing a massive JSON array of all open issues returned by an MCP server.

## Mitigating Rate Limits and API Exhaustion

POSIX applications inherently expect microsecond latency for file operations. If an agent decides to aggressively scan its environment by executing `grep -r "database error" /mnt/confluence_spaces/`, a naive FUSE daemon would translate this single command into tens of thousands of simultaneous HTTP GET requests. This behavior will instantly trigger API rate limiters, resulting in HTTP 429 Too Many Requests errors and temporary account suspensions.

To sustain throughput and prevent API exhaustion, the FUSE and Git layers must implement sophisticated mitigation strategies:

- **Sliding Window Counters and Token Buckets:** The FUSE daemon incorporates internal rate limiters utilizing Redis or local memory to queue and throttle outgoing requests, preventing burst traffic from exceeding the API's fixed window thresholds.
- **Asynchronous Background Fetching:** Implementations like the `jirafs` fetcher operate in the background, downloading issue properties asynchronously and caching them locally. When the agent lists a directory, the system serves the cached data instantly rather than querying the network synchronously.
- **Graceful Degradation via POSIX Errors:** If the API rate limit is exceeded, the FUSE daemon translates the HTTP 429 error into standard Unix I/O errors, such as `EIO` (Input/output error) or `ETIMEDOUT` (Connection timed out). The agent's shell environment natively understands these errors and can invoke exponential backoff strategies automatically without requiring specialized prompt engineering.

By buffering the aggressive POSIX reads through a caching layer and deferring writes to batched `git push` operations handled by the remote helper, the system maximizes the agent's operational speed while maintaining strict compliance with external API quotas.
