SRS Client
==========

[![srs-client](https://img.shields.io/badge/v0.3.0-blue) v0.3.0](https://github.com/fluxomnia-systems/srs-client/tree/v0.3.0) ([changelog](https://github.com/fluxomnia-systems/srs-client/blob/main/CHANGELOG.md))

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
srs-client = "0.3.0"
```

## Running tests

To run integration tests against a local SRS, start it via the project test helper and run the dedicated test target:

```bash
make test-http-api
```

This command starts a local SRS container, waits for the HTTP API endpoint, runs:

```bash
cargo test --test test_http_api
```

and then shuts the container down. You can override cargo test args with `CARGO_TEST_ARGS`, for example:

```bash
make test-http-api CARGO_TEST_ARGS='--test test_http_api -- --test kickoff_client'
```

You can override ports and image used by compose:

- `SRS_IMAGE` (default `ossrs/srs:v5`)
- `SRS_HTTP_API_HOST` (default `127.0.0.1`)
- `SRS_HTTP_API_PORT` (default `1985`)
- `SRS_RTMP_PORT` (default `1935`)
- `SRS_HTTP_SERVER_PORT` (default `8080`)

You can also keep the container after tests with `KEEP_SRS_COMPOSE=1`, or pass a custom compose file with `SRS_COMPOSE_FILE`.

Directly running `cargo test --test test_http_api` still requires `SRS_HTTP_API_URL` to be set.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For maintainers releasing new versions, see [RELEASE.md](RELEASE.md) for detailed release instructions.

## License

This project is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

[Rust]: https://www.rust-lang.org
[1]: https://github.com/ossrs/srs
[2]: https://crates.io/crates/srs-client
[3]: https://ossrs.io/lts/en-us/docs/v5/doc/http-api
[4]: https://docs.rs/srs-client/
