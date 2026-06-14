//! Request parser: methods to parse received string request

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
};

/// Returns the first line of the GET request header, or `None` if the
/// connection closed or no valid request line was received.
pub async fn parse_header(stream: &mut TcpStream) -> Option<String> {
    let mut buf_reader = BufReader::new(stream);
    let mut head_str = String::new();

    loop {
        match buf_reader.read_line(&mut head_str).await {
            Ok(0) => return None,
            Ok(count) => {
                if head_str.starts_with("GET") {
                    return Some(head_str);
                }
                if count < 3 {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }
}
