use std::error::Error;
use tokio;

use web_server::server::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Server::run("127.0.0.1:15400").await;

    Ok(())
}