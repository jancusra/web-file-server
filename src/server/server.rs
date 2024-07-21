use std::{
    io::BufReader,
    net::{TcpListener, TcpStream}
};

use crate::server::{request_parser, response};

pub struct Server { }

impl Server {
    pub async fn run(address_port: &str) {
        //let server_cfg = ServerConfig::init();

        match TcpListener::bind(address_port) {
            Ok(listener) => {
                println!("Server is running on http://{}", address_port);

                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => Self::handle_request(stream/*, &server_cfg*/).await,
                        Err(_) => {}
                    }
                }
            },
            Err(_) => {}
        }
    }

    async fn handle_request(mut stream: TcpStream/*, config: &ServerConfig*/) {
        let mut buffer_reader = BufReader::new(&mut stream);
        let _request_header = request_parser::parse_header(&mut buffer_reader);

        response::serve_file(&stream, &format!("{}{}", env!("CARGO_MANIFEST_DIR"), "\\src\\www\\index.html"), 
            &"text/html; charset=UTF-8", false);
    }
}