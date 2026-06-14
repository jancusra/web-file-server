//! Response: methods for formatting and returning the response

use std::io::Result;

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Create a response file with a header and write the result to the stream
pub async fn serve_file(
    stream: &mut TcpStream,
    file_path: &str,
    content_type: &str,
    cache: bool,
) -> Result<()> {
    let content = get_file_as_byte_vec(file_path).await?;
    let header = get_response_header(content_type, content.len(), cache);

    let mut response = header.into_bytes();
    response.extend(content);

    stream.write_all(&response).await?;
    stream.flush().await?;

    Ok(())
}

/// Write a minimal 404 response to the stream (errors are logged, not propagated)
pub async fn serve_not_found(stream: &mut TcpStream) {
    let body = b"404 Not Found";
    let header = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=UTF-8\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );

    let mut response = header.into_bytes();
    response.extend_from_slice(body);

    if let Err(error) = stream.write_all(&response).await {
        eprintln!("Failed to write 404 response: {error}");
    }
}

/// Prepare a response header with OK status
pub fn get_response_header(content_type: &str, content_length: usize, cache: bool) -> String {
    let mut cache_str = "".to_string();

    if !cache {
        cache_str = "Cache-Control: no-store\r\n".to_string();
    }

    format!("HTTP/1.1 200 OK\r\n{cache_str}Content-Type: {content_type}\r\nContent-Length: {content_length}\r\n\r\n")
}

/// Convert a specific file to a vector of bytes
pub async fn get_file_as_byte_vec(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(filename).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    Ok(buffer)
}
