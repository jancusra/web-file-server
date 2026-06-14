# Basic server serving web files in Rust

The basic server is focused to return all required files to display the main web page.
The code structure is focused on the functionality of a tiny server, it is definitely not final and usable for larger web applications.

It is built on an async [tokio](https://tokio.rs/) runtime, serves a fixed whitelist of
static files (so arbitrary paths can't be read), and handles each connection concurrently.

### How to run the server

Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed, then:

```sh
cargo run
```

By default the server listens on `http://127.0.0.1:15400` — open that address in your browser.

### Configuration

Both the listen address and the web root can be overridden via environment variables:

| Variable      | Default                    | Description                          |
| ------------- | -------------------------- | ------------------------------------ |
| `SERVER_ADDR` | `127.0.0.1:15400`          | Address and port the server binds to |
| `WEB_ROOT`    | `<crate>/src/www`          | Directory the served files live in   |

Example:

```sh
SERVER_ADDR=0.0.0.0:8080 WEB_ROOT=./public cargo run
```

### Running the tests

```sh
cargo test
```
