//! Server: define basic methods to start the server and process requests

use std::{
    io::BufReader,
    net::{TcpListener, TcpStream}
};

use crate::server::{configuration::ServerConfig, request_parser, response};

/// Main server instance
pub struct Server { }

impl Server {
    // Starting a server instance
    pub async fn run(address_port: &str) {
        let server_cfg = ServerConfig::init();

        match TcpListener::bind(address_port) {
            Ok(listener) => {
                println!("Server is running on http://{}", address_port);

                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => Self::handle_request(stream, &server_cfg).await,
                        Err(_) => {}
                    }
                }
            },
            Err(_) => {}
        }
    }

    // Processing a specific request as a stream
    async fn handle_request(mut stream: TcpStream, config: &ServerConfig) {
        let mut buffer_reader = BufReader::new(&mut stream);
        let request_header = request_parser::parse_header(&mut buffer_reader);

        if let (Some(file_path), Some(file_entry)) = config.get_file_data(&request_header) {
            response::serve_file(&stream, &file_path, &file_entry.content_type, file_entry.cache);
        }

        response::serve_file(&stream, &config.default_file, &config.default_content_type, false);
    }
}