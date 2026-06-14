use std::error::Error;

use web_server::server::instance::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Listen address: overridable via the SERVER_ADDR env var
    let address = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:15400".to_string());

    // starting a web server on a local IP address
    Server::new().run(&address).await;

    Ok(())
}
