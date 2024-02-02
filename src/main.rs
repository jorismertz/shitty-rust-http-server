use http_server::ThreadPool;
mod res;
pub use crate::res::response;

use std::{str::FromStr, thread};

pub use std::{
    error::Error,
    fmt::{format, write, Display},
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let port = 8070;
    let hostname = "localhost";

    let address = format!("{}:{}", hostname, port);
    let listener = TcpListener::bind(&address).expect(&format!(
        "Could not bind on address {}, Is this port already in use?",
        address
    ));

    println!("Listening on {}", address);

    // I hope this is enough to serve my html files
    let pool = ThreadPool::new(16);

    for stream in listener.incoming() {
        let stream = stream.expect("Failed to establish incoming connection");

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

#[derive(Debug, PartialEq)]
enum Method {
    GET,
    POST,
    PUT,
    OPTIONS,
    DELETE,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(input: &str) -> Result<Method, Self::Err> {
        match input.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "OPTIONS" => Ok(Method::OPTIONS),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct Request {
    method: Method,
    path: String,
    protocol_version: String,
    headers: Vec<(String, String)>,
}

fn parse_request(req: Vec<String>) -> Option<Request> {
    if req.len() < 1 {
        return None;
    }

    let first_line = req[0].split(" ").collect::<Vec<&str>>();
    if first_line.len() < 3 {
        return None;
    }

    let mut headers: Vec<(String, String)> = Vec::new();
    for line in req.clone().into_iter().skip(1) {
        let split: Vec<&str> = line.split(": ").collect();

        // I'm just assuming this is how http works...
        if split.len() != 2 {
            continue;
        }

        headers.push((split[0].to_string(), split[1].to_string()));
    }

    return Some(Request {
        method: Method::from_str(first_line.get(0).unwrap()).unwrap(),
        path: first_line.get(1).unwrap().to_string(),
        protocol_version: first_line.get(2).unwrap().to_string(),
        headers,
    });
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let request = parse_request(http_request).unwrap();

    dbg!(&request.path);

    match request.path.as_str() {
        "/" => {
            let contents = fs::read_to_string("./src/pages/index.html").unwrap();
            stream
                .write_response_body(&contents, &response::Status::Ok)
                .unwrap();
        }
        _ => {
            let contents = fs::read_to_string("./src/pages/404.html").unwrap();
            stream
                .write_response_body(&contents, &response::Status::NotFound)
                .unwrap();
        }
    }
}

trait HttpResponses {
    fn write_response_body(&mut self, html: &str, response: &response::Status) -> Result<(), ()>;
}

impl HttpResponses for TcpStream {
    fn write_response_body(&mut self, html: &str, status: &response::Status) -> Result<(), ()> {
        let msg = status.response_string();
        let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", msg, html.len(), html);
        self.write_all(response.as_bytes()).unwrap();

        return Ok(());
    }
}
