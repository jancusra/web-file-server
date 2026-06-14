//! # Basic server serving web files in Rust
//!
//! A tiny async (tokio) server that returns a fixed whitelist of static files
//! needed to display the main web page.
//!
//! Run with `cargo run` and open the printed address in a browser. The listen
//! address and the web root can be overridden via the `SERVER_ADDR` and
//! `WEB_ROOT` environment variables.

pub mod server;
