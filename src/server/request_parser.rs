//! Request parser: methods to parse received string request

use std::time::Duration;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
    time::timeout,
};

/// How long to wait for the client to send the request line before giving up.
const REQUEST_LINE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum accepted length of the request line, to bound memory per connection.
const MAX_REQUEST_LINE: u64 = 8 * 1024;

/// Returns the first line of the GET request, or `None` if the connection
/// closed, timed out, exceeded the size limit, or sent no valid request line.
///
/// Only the request line is needed to route a static file, so we read exactly
/// one (length-capped) line instead of draining the whole header block.
pub async fn parse_header(stream: &mut TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream).take(MAX_REQUEST_LINE);
    let mut request_line = String::new();

    match timeout(REQUEST_LINE_TIMEOUT, reader.read_line(&mut request_line)).await {
        // Read a non-empty line that is a GET request
        Ok(Ok(count)) if count > 0 && request_line.starts_with("GET") => Some(request_line),
        // Connection closed, non-GET request, read error, or timeout
        _ => None,
    }
}
