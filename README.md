SRS Client
==========

[![srs-client](https://img.shields.io/badge/v0.4.0-blue) v0.4.0](https://github.com/fluxomnia-systems/srs-client/tree/v0.4.0) ([changelog](https://github.com/fluxomnia-systems/srs-client/blob/main/CHANGELOG.md))

The [SRS (Simple RTMP Server)][1] [Rust] Client or [srs-client][2] is a [Rust] package that provides bindings for the main functionalities of the SRS server. It supports two modes of operation:

1. As an HTTP client to interact with the SRS HTTP API
2. For handling SRS HTTP callbacks

## HTTP Client Mode

In this mode, srs-client uses HTTP to communicate with the server based on the [SRS HTTP API][3] specification.

### Usage

To use srs-client as an HTTP client:

```rust
use srs_client::{SrsClient, SrsClientError, SrsClientResp};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url = env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    
    // Get SRS version
    let result: Result<SrsClientResp, SrsClientError> = client.get_version().await;
    println!("SRS version: {:?}", result);

    // Get streams
    let result: Result<SrsClientResp, SrsClientError> = client.get_streams().await;
    println!("Streams: {:?}", result);

    Ok(())
}
```

## HTTP Callback Mode

In this mode, srs-client provides structs to handle callbacks sent by SRS.

### Usage

To handle SRS callbacks:

```rust
use actix_web::{post, web};
use srs_client::{SrsCallbackEvent, SrsCallbackReq};

#[post("srs_callback")]
pub async fn on_callback(req: web::Json<SrsCallbackReq>) -> Result<&'static str, String> {
    match req.action {
        SrsCallbackEvent::OnConnect => {
            // Handle connection event
            dbg!(&req);
            Ok(())
        }
        _ => Ok(()),
    }
    .map(|()| "0")
}
```

## Features

- HTTP Client Mode:
  - Retrieve server information (version, vhosts, streams, clients)
  - Monitor system stats (rusages, self_proc_stats, system_proc_stats, meminfos)
  - Manage clients (kickoff)

- HTTP Callback Mode:
  - Handle various SRS events (OnConnect, OnPublish, etc.)

Full API Reference is available [here][4].

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
srs-client = "0.4.0"
```

## Supported SRS API

The client targets SRS **6.0.191** (stable), **7.0.162** (alpha), and **8.0.48**
(development). CI runs contract tests and live management, RTMP publishing, and
WebRTC signaling tests against each pinned image.

- Monitoring: version, discovery, summaries, process/memory statistics, features,
  authors, request diagnostics, vhosts, streams, clients, and pagination.
- Management: client kickoff, RAW API configuration, reload and reload status,
  origin-cluster queries, and Prometheus metrics.
- Optional diagnostics: tcmalloc JSON/text statistics, process signals, Valgrind,
  and RTC NACK simulation. These require the corresponding server build/config.
- Signaling: JSON RTC publish/play, WHIP publishing, WHEP playback, and session
  deletion using the returned Location and token.
- Callbacks: all JSON hook events, including `on_close` and forwarding-backend
  `on_forward`, event-specific fields, and preservation of future payload fields.
  `SrsForwardResponse` supplies the forwarding URL response. `on_hls_notify` is a
  separate HTTP GET notification that your application must serve.

This crate handles HTTP control and signaling, not ICE/DTLS/RTP media transport.
WHIP/WHEP PATCH/trickle ICE is not implemented because the inspected SRS handlers
provide SDP exchange and DELETE, not PATCH negotiation. Optional build diagnostics
are contract-tested; standard SRS images correctly report them as unavailable.

### Management and transport configuration

Methods borrow the client, so the same connection pool can be reused:

```rust,no_run
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
use srs_client::SrsClient;
let http = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()?;
let client = SrsClient::with_http_client("http://localhost:1985", http)?;
let info = client.get_request_info().await?;
let config = client.get_raw_config().await?;
let metrics = client.get_metrics().await?; // requires exporter.enabled
# Ok(())
# }
```

Use a configured reqwest client for default authentication headers, custom TLS
roots, proxies, or timeouts. The URL identifies the SRS origin; API paths are rooted
at `/api/v1/`, `/rtc/v1/`, and `/metrics` rather than a reverse-proxy path prefix.
Query parameters are URL-encoded. SRS cluster responses can echo encoded values.

### WHIP/WHEP

```rust,no_run
# async fn example(offer_sdp: &str) -> Result<(), Box<dyn std::error::Error>> {
use srs_client::SrsClient;
let client = SrsClient::build("http://localhost:1985")?;
let session = client.whip(offer_sdp, &[("app", "live"), ("stream", "camera")]).await?;
// Pass session.sdp to your WebRTC peer implementation.
client.delete_session(&session).await?;
# Ok(())
# }
```

Use `whep` for playback or `rtc_publish`/`rtc_play` with `RtcRequest` for SRS JSON
signaling. Query options accept server-supported candidate, codec, and stream
parameters. Session deletion rejects URLs outside the configured SRS RTC origin.

### Migrating from 0.3 to 0.4

Version 0.4.0 changes public models and error behavior:

- Nonzero SRS body codes now return `SrsClientError::ApiError(code)`, including
  failed kickoff calls. Typed item getters return `None` only for that resource's
  specific not-found code; other failures propagate.
- `get_requests()` now returns `Requests` from `/api/v1/tests/requests`.
  `get_configs()` now returns `Raw` from `/api/v1/raw?rpc=raw`.
- Use `get_request_info()` and `get_raw_config()` instead of the deprecated
  `get_requests_summary()` and `get_configs_summary()` helpers.
- The old `get_perf*`, `get_dvr*`, and `get_tcmalloc_summary()` helpers are deprecated
  and return `UnsupportedEndpoint`. Use metrics, DVR callbacks, and
  `get_tcmalloc_stats()` respectively. A server's discovery-index fallback is never
  accepted as successful diagnostic data.
- `SrsClientRespData` and `SrsCallbackEvent` have additional variants. Callback
  structs have additional optional fields and an `extra` map; update exhaustive
  matches and struct literals. Version fields are now public.
- Unknown or malformed response shapes fail decoding instead of becoming empty
  cluster responses.

## Running tests

Run offline fixtures and local mock-server contracts without an SRS instance:

```bash
cargo test --all-targets
```

Live tests are explicitly ignored by default. Docker Compose, curl, and ffmpeg are
required to run the full suite against an isolated SRS instance:

```bash
make test-http-api
SRS_IMAGE=ossrs/srs:8.0.48 SRS_EXPECTED_VERSION=8.0.48 make test-http-api
```

The helper enables metrics and RAW reload, starts SRS, waits for readiness, runs all
tests including the live tests, and removes its Compose services afterward. Live
media tests publish synthetic H264/AAC, read stream/client metadata, and kick off
the publisher. SDP tests negotiate and delete WHIP/WHEP sessions and negotiate
JSON RTC sessions; they do not certify end-to-end WebRTC media delivery.

For an already running **test** server:

```bash
SRS_HTTP_API_URL=http://localhost:1985 \
SRS_RTMP_URL=rtmp://localhost:1935/live \
cargo test --test test_live -- --ignored --test-threads=1
```

The live tests reload configuration and disconnect only their synthetic publisher.
The server must enable its exporter, RAW reload, and RTC support. Standard-image
optional-feature checks expect tcmalloc, signal, Valgrind, and NACK simulation to be
unavailable. JSON RTC sessions without a completed handshake expire in SRS.

Compose overrides: `SRS_IMAGE` (default `ossrs/srs:6.0.191`), `SRS_HTTP_API_HOST`,
`SRS_HTTP_API_PORT`, `SRS_RTMP_PORT`, `SRS_HTTP_SERVER_PORT`, and `SRS_COMPOSE_FILE`.
Set `KEEP_SRS_COMPOSE=1` to retain the test instance. `CARGO_TEST_ARGS` replaces the
default cargo arguments, for example `--test test_contract`.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For maintainers releasing new versions, see [RELEASE.md](RELEASE.md) for detailed release instructions.

## License

This project is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

[Rust]: https://www.rust-lang.org
[1]: https://github.com/ossrs/srs
[2]: https://crates.io/crates/srs-client
[3]: https://ossrs.io/lts/en-us/docs/v6/doc/http-api
[4]: https://docs.rs/srs-client/
