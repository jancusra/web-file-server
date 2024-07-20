use std::net::TcpListener;

pub struct Server { }

impl Server {
    pub async fn run(address_port: &str) {
        //let server_cfg = ServerConfig::init();

        match TcpListener::bind(address_port) {
            Ok(listener) => {
                println!("Server is running on http://{}", address_port);

                for stream in listener.incoming() {
                    match stream {
                        Ok(_stream) => {},
                        Err(_) => {}
                    }
                }
            },
            Err(_) => {}
        }
    }
}