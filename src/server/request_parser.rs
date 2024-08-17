//! Request parser: methods to parse received string request

use std::{
    io::{prelude::*, BufReader},
    net::TcpStream
};

/// Returns the first line of the GET request header
pub fn parse_header(buf_reader: &mut BufReader<&mut TcpStream>) -> String
{
    let mut head_str = String::new();

    loop {
        match buf_reader.read_line(&mut head_str) {
            Ok(count) => {
                if head_str.starts_with("GET") {
                    return head_str;
                }
                if count < 3 {
                    break;
                }
            },
            Err(_) => return head_str
        }
    }

    head_str
}