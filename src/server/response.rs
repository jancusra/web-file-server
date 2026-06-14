//! Response: methods for formatting and returning the response

use std::io::Result;

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Write a 200 response with the given body to the stream.
///
/// The body is read by the caller first, so a missing file never reaches this
/// function: any error here means a partial response is already on the wire and
/// the connection should simply be dropped (no fallback 404).
pub async fn write_response(
    stream: &mut TcpStream,
    content_type: &str,
    body: &[u8],
    cache: bool,
) -> Result<()> {
    let header = get_response_header(content_type, body.len(), cache);

    let mut response = header.into_bytes();
    response.extend_from_slice(body);

    stream.write_all(&response).await?;
    stream.flush().await?;

    Ok(())
}

/// Write a minimal 404 response to the stream (errors are logged, not propagated).
///
/// This is a safety net rather than a path an arbitrary client can hit: requests
/// are routed against an in-memory whitelist, so this only fires when a
/// whitelisted file is missing or unreadable on disk at request time.
pub async fn serve_not_found(stream: &mut TcpStream) {
    let body = b"404 Not Found";
    let header = format!(
        "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );

    let mut response = header.into_bytes();
    response.extend_from_slice(body);

    if let Err(error) = stream.write_all(&response).await {
        eprintln!("Failed to write 404 response: {error}");
    }
}

/// Prepare a response header with OK status.
///
/// Every connection serves a single response and is then closed, so we always
/// advertise `Connection: close` rather than (incorrectly) implying keep-alive.
pub fn get_response_header(content_type: &str, content_length: usize, cache: bool) -> String {
    let cache_str = if cache {
        ""
    } else {
        "Cache-Control: no-store\r\n"
    };

    format!("HTTP/1.1 200 OK\r\nConnection: close\r\n{cache_str}Content-Type: {content_type}\r\nContent-Length: {content_length}\r\n\r\n")
}

/// Convert a specific file to a vector of bytes
pub async fn get_file_as_byte_vec(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(filename).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_has_status_type_and_length() {
        let header = get_response_header("text/css", 42, true);

        assert!(header.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(header.contains("Content-Type: text/css\r\n"));
        assert!(header.contains("Content-Length: 42\r\n"));
        assert!(header.ends_with("\r\n\r\n"));
    }

    #[test]
    fn cache_control_present_only_when_not_cached() {
        assert!(get_response_header("text/css", 1, false).contains("Cache-Control: no-store\r\n"));
        assert!(!get_response_header("font/woff", 1, true).contains("Cache-Control"));
    }
}
