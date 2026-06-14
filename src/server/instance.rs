//! Server: define basic methods to start the server and process requests

use std::io::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

use crate::server::{configuration::ServerConfig, request_parser, response};

/// Main server instance
pub struct Server {
    config: Arc<ServerConfig>,
}

impl Server {
    // Create a new server instance bound to the given address, with otherwise
    // default configuration.
    pub fn new(address: String) -> Self {
        Self::new_with_config(ServerConfig::new(address))
    }

    /// Create a server instance from a custom configuration, so several servers
    /// with different settings (e.g. `max_connections`) can run side by side.
    pub fn new_with_config(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    // Bind to the configured address and start serving requests.
    //
    // Takes `self` by value so the returned future is `'static` and each server
    // can be driven on its own task (e.g. `tokio::spawn(server.run())`) when
    // running several instances side by side.
    //
    // Returns the bind error so the caller can exit with a non-zero status when
    // the server fails to start.
    pub async fn run(self) -> Result<()> {
        let address = &self.config.address;

        let listener = TcpListener::bind(address).await.map_err(|error| {
            eprintln!("Failed to bind to {address}: {error}");
            error
        })?;

        println!("Server is running on http://{address}");

        // Serve until interrupted (Ctrl-C), then stop accepting new connections
        // and return so the process can exit cleanly.
        tokio::select! {
            _ = self.serve(listener) => {}
            _ = tokio::signal::ctrl_c() => println!("\nShutting down"),
        }

        Ok(())
    }

    /// Accept connections on an already-bound listener, handling each one
    /// concurrently so a slow client can't block the others. The number of
    /// in-flight connections is capped by [`ServerConfig::max_connections`].
    ///
    /// Split out from [`Server::run`] so tests can bind to an ephemeral port
    /// (`127.0.0.1:0`) and drive a real connection against the server.
    pub async fn serve(&self, listener: TcpListener) {
        let limit = Arc::new(Semaphore::new(self.config.max_connections));

        loop {
            // Reserve a slot before accepting, so we never spawn more than
            // `max_connections` handler tasks at once.
            let permit = match Arc::clone(&limit).acquire_owned().await {
                Ok(permit) => permit,
                // The semaphore is never closed, so this is unreachable.
                Err(_) => return,
            };

            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let config = Arc::clone(&self.config);
                    tokio::spawn(async move {
                        Self::handle_request(stream, config).await;
                        // Release the slot once the connection is done.
                        drop(permit);
                    });
                }
                Err(error) => eprintln!("Failed to accept connection: {error}"),
            }
        }
    }

    // Processing a specific request as a stream
    async fn handle_request(mut stream: TcpStream, config: Arc<ServerConfig>) {
        let request_header =
            match request_parser::parse_header(&mut stream, config.header_timeout).await {
                Some(header) => header,
                None => return,
            };

        // Resolve the target: a whitelisted file, or the default index otherwise.
        // Both are paths relative to the web root.
        let (relative, content_type, cache) = match config.get_file_data(&request_header) {
            Some(served) => (
                served.rel_path.as_str(),
                served.content_type.as_str(),
                served.cache,
            ),
            None => (
                config.default_file.as_str(),
                config.default_content_type.as_str(),
                false,
            ),
        };

        // Resolve against the configured web root at request time, so changing
        // `web_path` (e.g. per instance) takes effect here.
        let path = config.file_path(relative);

        // Read the body before writing anything. A missing/unreadable file is
        // turned into a 404 here, while the stream is still untouched, so we can
        // never emit a half-written 200 followed by a second response.
        let body = match response::get_file_as_byte_vec(&path).await {
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
