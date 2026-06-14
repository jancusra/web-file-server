//! Request parser: methods to parse received string request

use std::time::Duration;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
    time::timeout,
};

/// How long to wait for the client while reading the request headers.
const HEADER_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum accepted size of the request header block, to bound memory per
/// connection.
const MAX_HEADER_BYTES: u64 = 16 * 1024;

/// Returns the request line of the GET request, or `None` if the connection
/// closed, timed out, exceeded the size limit, or sent no valid request line.
///
/// Only the request line is needed to route a static file, but the rest of the
/// header block is still drained (see [`drain_headers`]) so the whole request
/// is consumed before we reply and close the connection.
pub async fn parse_header(stream: &mut TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream).take(MAX_HEADER_BYTES);
    let mut request_line = String::new();

    match timeout(HEADER_TIMEOUT, reader.read_line(&mut request_line)).await {
        // Read a non-empty line that is a GET request
        Ok(Ok(count)) if count > 0 && request_line.starts_with("GET") => {}
        // Connection closed, non-GET request, read error, or timeout
        _ => return None,
    }

    // Best-effort: consume the remaining request headers. Bytes left unread in
    // the socket can otherwise trigger a TCP reset on close that truncates the
    // response on some clients (notably Windows). A client that never sends the
    // terminating blank line can't hold the connection hostage — the drain is
    // both size- and time-bounded, and its outcome is intentionally ignored.
    let _ = timeout(HEADER_TIMEOUT, drain_headers(&mut reader)).await;

    Some(request_line)
}

/// Read and discard request header lines until the blank line that terminates
/// the header block (or EOF / the size cap of the underlying reader).
async fn drain_headers<R: AsyncBufReadExt + Unpin>(reader: &mut R) {
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            // EOF, or the empty line that ends the header block
            Ok(0) => break,
            Ok(_) if line == "\r\n" || line == "\n" => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
}
