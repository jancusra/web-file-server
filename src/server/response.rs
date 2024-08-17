//! Response: methods for formatting and returning the response

use std::{
    fs::File,
    io::{prelude::*, BufReader},
    net::TcpStream
};

/// Create a response file with a header and write the result to the stream
pub fn serve_file(mut stream: &TcpStream, file_path: &str, content_type: &str, cache: bool) {
    let content = get_file_as_byte_vec(file_path);
    let header = get_response_header(content_type, content.len(), cache);
    let mut header_bytes = header.into_bytes();

    for byte in content {
        header_bytes.push(byte);
    }

    stream.write_all(header_bytes.as_slice()).ok();
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
pub fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut buffer = Vec::new();

    match File::open(filename) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            reader.read_to_end(&mut buffer).ok();
        },
        Err(_) => {}
    }

    buffer
}