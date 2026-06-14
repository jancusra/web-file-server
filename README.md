# Basic server serving web files in Rust

[![all tests](https://github.com/jancusra/web-file-server/actions/workflows/rust.yml/badge.svg)](https://github.com/jancusra/web-file-server/actions/workflows/rust.yml)

The basic server is focused to return all required files to display the main web page.
The code structure is focused on the functionality of a tiny server, it is definitely not final and usable for larger web applications.

It is built on an async [tokio](https://tokio.rs/) runtime, serves a fixed whitelist of
static files (so arbitrary paths can't be read), and handles connections concurrently up
to a configurable limit (1024 by default) so a flood of clients can't spawn unbounded work.

### How to run the server

Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed, then:

```sh
cargo run
```

By default the server listens on `http://127.0.0.1:15400` — open that address in your browser.
Press `Ctrl-C` to stop the server; it stops accepting new connections and shuts down cleanly.

### Configuration

The server is configured in Rust code, not through environment variables. The
listen address is passed to `ServerConfig::new`; the remaining settings are plain
fields with sensible defaults — the connection limit, the request-header timeout,
and `web_path` (the directory files are served from, defaulting to the bundled
`src/www`). Build a config and start the server with `Server::new_with_config`:

```rust
use std::time::Duration;
use web_server::server::{configuration::ServerConfig, instance::Server};

let mut config = ServerConfig::new("0.0.0.0:8080".to_string());
config.max_connections = 4096;
config.header_timeout = Duration::from_secs(10);
config.web_path = "./public".to_string(); // serve a different directory
Server::new_with_config(config).run().await?;
```

The default entry point (`src/main.rs`) simply binds `127.0.0.1:15400` via
`Server::new`, which serves the bundled web root.

### Running several instances

Each `Server` is independent, so multiple instances can run side by side on
different addresses, with different limits or even different content. `run`
takes `self`, so each one can be driven on its own task:

```rust
let a = Server::new("127.0.0.1:8080".to_string());

let mut config_b = ServerConfig::new("127.0.0.1:9090".to_string());
config_b.web_path = "./other-site".to_string();
let b = Server::new_with_config(config_b);

tokio::spawn(a.run());
tokio::spawn(b.run());
```

### Running the tests

```sh
cargo test
```

This runs both the unit tests (request parsing, MIME lookup, response headers)
and the integration tests, which start the server on an ephemeral port and drive
real TCP requests against it.
