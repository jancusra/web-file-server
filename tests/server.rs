//! Integration tests: drive a real TCP connection against a running server.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

use web_server::server::{configuration::ServerConfig, instance::Server};

/// Start the default server on an ephemeral port and return its address.
async fn spawn_server() -> SocketAddr {
    // The address is unused here: spawn() binds its own ephemeral listener.
    spawn(Server::new("127.0.0.1:0".to_string())).await
}

/// Start the given server on an ephemeral port and return its address.
async fn spawn(server: Server) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("should bind to an ephemeral port");
    let addr = listener.local_addr().expect("should have a local address");

    tokio::spawn(async move {
        server.serve(listener).await;
    });

    addr
}

/// Send a GET request for `path` and return the full raw response.
async fn get(addr: SocketAddr, path: &str) -> String {
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n");
    send_request(addr, &request).await
}

/// Send a raw request line + headers and return the full raw response.
async fn send_request(addr: SocketAddr, request: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.expect("should connect");

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
    assert!(response.contains("Content-Type: text/css; charset=UTF-8\r\n"));
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

#[tokio::test]
async fn runs_with_a_custom_config() {
    // A server built from a custom config (here a tiny connection cap) still
    // serves normally — exercises the Server::new_with_config path.
    let mut config = ServerConfig::new("127.0.0.1:0".to_string());
    config.max_connections = 1;
    let addr = spawn(Server::new_with_config(config)).await;

    let response = get(addr, "/styles.css").await;
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
}

#[tokio::test]
async fn serves_content_from_a_custom_web_root() {
    use std::fs;

    // A temporary web root with a distinctive index page, so we can tell it
    // apart from the crate's bundled src/www.
    let dir = std::env::temp_dir().join(format!("web-server-custom-root-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("should create temp web root");
    let marker = "<h1>served from a custom web root</h1>";
    fs::write(dir.join("index.html"), marker).expect("should write index.html");

    let mut config = ServerConfig::new("127.0.0.1:0".to_string());
    config.web_path = dir.to_string_lossy().into_owned();
    let addr = spawn(Server::new_with_config(config)).await;

    // Any unknown path falls back to the index served from the custom root.
    let response = get(addr, "/whatever").await;
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response.contains(marker));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn rejects_non_get_method() {
    let addr = spawn_server().await;

    // A method that isn't GET (and a GET look-alike) is dropped without a reply.
    for request in [
        "POST /styles.css HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GETX /styles.css HTTP/1.1\r\nHost: localhost\r\n\r\n",
    ] {
        let response = send_request(addr, request).await;
        assert!(
            response.is_empty(),
            "expected the connection to close with no response, got: {response:?}"
        );
    }
}
