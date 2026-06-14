//! Server: define basic methods to start the server and process requests

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use crate::server::{configuration::ServerConfig, request_parser, response};

/// Main server instance
pub struct Server {}

impl Server {
    // Starting a server instance
    pub async fn run(address_port: &str) {
        let server_cfg = Arc::new(ServerConfig::init());

        let listener = match TcpListener::bind(address_port).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("Failed to bind to {address_port}: {error}");
                return;
            }
        };

        println!("Server is running on http://{}", address_port);

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    // Handle each connection concurrently so a slow client
                    // can't block the others.
                    let config = Arc::clone(&server_cfg);
                    tokio::spawn(async move {
                        Self::handle_request(stream, config).await;
                    });
                }
                Err(error) => eprintln!("Failed to accept connection: {error}"),
            }
        }
    }

    // Processing a specific request as a stream
    async fn handle_request(mut stream: TcpStream, config: Arc<ServerConfig>) {
        let request_header = match request_parser::parse_header(&mut stream).await {
            Some(header) => header,
            None => return,
        };

        let result =
            if let (Some(file_path), Some(file_entry)) = config.get_file_data(&request_header) {
                response::serve_file(
                    &mut stream,
                    &file_path,
                    &file_entry.content_type,
                    file_entry.cache,
                )
                .await
            } else {
                response::serve_file(
                    &mut stream,
                    &config.default_file,
                    &config.default_content_type,
                    false,
                )
                .await
            };

        if let Err(error) = result {
            eprintln!("Error serving request: {error}");
            response::serve_not_found(&mut stream).await;
        }
    }
}
