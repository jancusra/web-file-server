//! Server: define basic methods to start the server and process requests

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

use crate::server::{configuration::ServerConfig, request_parser, response};

/// Main server instance
pub struct Server {
    config: Arc<ServerConfig>,
}

impl Server {
    // Create a new server instance with the default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(ServerConfig::init()),
        }
    }

    // Bind to the given address and start serving requests
    pub async fn run(&self, address_port: &str) {
        let listener = match TcpListener::bind(address_port).await {
            Ok(listener) => listener,
            Err(error) => {
                eprintln!("Failed to bind to {address_port}: {error}");
                return;
            }
        };

        println!("Server is running on http://{}", address_port);

        self.serve(listener).await;
    }

    /// Accept connections on an already-bound listener, handling each one
    /// concurrently so a slow client can't block the others.
    ///
    /// Split out from [`Server::run`] so tests can bind to an ephemeral port
    /// (`127.0.0.1:0`) and drive a real connection against the server.
    pub async fn serve(&self, listener: TcpListener) {
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let config = Arc::clone(&self.config);
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

        // Resolve the target: a whitelisted file, or the default index otherwise.
        let (path, content_type, cache) = match config.get_file_data(&request_header) {
            Some(served) => (
                served.fs_path.as_str(),
                served.content_type.as_str(),
                served.cache,
            ),
            None => (
                config.default_file.as_str(),
                config.default_content_type.as_str(),
                false,
            ),
        };

        // Read the body before writing anything. A missing/unreadable file is
        // turned into a 404 here, while the stream is still untouched, so we can
        // never emit a half-written 200 followed by a second response.
        let body = match response::get_file_as_byte_vec(path).await {
            Ok(body) => body,
            Err(error) => {
                eprintln!("Error reading '{path}': {error}");
                response::serve_not_found(&mut stream).await;
                return;
            }
        };

        // Past this point the response is already (partially) on the wire, so a
        // write failure is only logged; a fallback response would corrupt it.
        if let Err(error) = response::write_response(&mut stream, content_type, &body, cache).await
        {
            eprintln!("Error writing response: {error}");
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
