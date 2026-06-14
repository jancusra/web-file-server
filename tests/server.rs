//! Integration tests: drive a real TCP connection against a running server.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

use web_server::server::instance::Server;

/// Start a server on an ephemeral port and return the address it's listening on.
async fn spawn_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("should bind to an ephemeral port");
    let addr = listener.local_addr().expect("should have a local address");

    let server = Server::new();
    tokio::spawn(async move {
        server.serve(listener).await;
    });

    addr
}

/// Send a GET request for `path` and return the full raw response.
async fn get(addr: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.expect("should connect");

    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .await
        .expect("should send request");
    stream.flush().await.expect("should flush request");

    // The server sends `Connection: close`, so the read completes at EOF.
    let mut response = Vec::new();
    timeout(Duration::from_secs(5), stream.read_to_end(&mut response))
        .await
        .expect("response should arrive before timeout")
        .expect("should read response");

    String::from_utf8_lossy(&response).into_owned()
}

#[tokio::test]
async fn serves_whitelisted_css_with_no_store() {
    let addr = spawn_server().await;
    let response = get(addr, "/styles.css").await;

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains("Connection: close\r\n"));
    assert!(response.contains("Content-Type: text/css\r\n"));
    // styles.css is marked non-cacheable
    assert!(response.contains("Cache-Control: no-store\r\n"));
    // body of the actual served file
    assert!(response.contains("@font-face"));
}

#[tokio::test]
async fn serves_cacheable_font_and_strips_query_string() {
    let addr = spawn_server().await;
    // The query string must be ignored when matching the whitelist.
    let response = get(addr, "/fonts/web-font.woff2?v=002").await;

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains("Content-Type: font/woff2\r\n"));
    // fonts are cacheable, so no Cache-Control header is sent
    assert!(!response.contains("Cache-Control"));
}

#[tokio::test]
async fn unknown_path_falls_back_to_index() {
    let addr = spawn_server().await;
    let response = get(addr, "/does-not-exist").await;

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains("Content-Type: text/html"));
}
