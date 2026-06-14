use std::error::Error;

use web_server::server::instance::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Listen address the server binds to
    Server::new("127.0.0.1:15400".to_string()).run().await?;

    Ok(())
}
