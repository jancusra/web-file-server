use std::error::Error;
use tokio;

use web_server::server::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // starting a web server on a local IP address
    Server::run("127.0.0.1:15400").await;

    Ok(())
}