use http_server::ThreadPool;
use serde::{Deserialize, Serialize};
mod res;
pub use crate::res::response;

use chrono::{DateTime, Utc};
use std::str::FromStr;

pub use std::{
    error::Error,
    fmt::{format, write, Display},
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    // let hostname = "192.168.0.101";
    let hostname = "localhost";
    let port = 8070;

    let address = format!("{hostname}:{port}");
    let listener = TcpListener::bind(&address).expect(&format!(
        "Could not bind on {address}, Is this port already in use?",
    ));

    println!("Listening on {address}");

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
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(input: &str) -> Result<Method, Self::Err> {
        match input.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
struct Request {
    method: Method,
    path: String,
    protocol_version: String,
    headers: response::Headers,
    body: Option<Vec<u8>>,
}

fn parse_request(mut stream: &mut TcpStream) -> Option<Request> {
    let mut buf_reader = BufReader::new(&mut stream);

    let req: Vec<_> = buf_reader
        .by_ref()
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if req.len() < 1 {
        return None;
    }

    let first_line = req[0].split(" ").collect::<Vec<&str>>();
    if first_line.len() < 3 {
        return None;
    }

    let mut headers: response::Headers = Vec::new();
    for line in req.clone().into_iter().skip(1) {
        let split: Vec<&str> = line.split(": ").collect();

        // I'm just assuming this is how http works...
        if split.len() != 2 {
            continue;
        }

        headers.push((split[0].to_string(), split[1].to_string()));
    }

    let mut body: Option<Vec<u8>> = None;

    let content_length_header = headers
        .clone()
        .into_iter()
        .find(|h| h.0.to_lowercase() == "content-length");

    if let Some(content_length) = content_length_header {
        let parsed_index = content_length.1.parse::<usize>().unwrap();
        let mut parsed_body = vec![0; parsed_index];

        match buf_reader.read_exact(&mut parsed_body) {
            Ok(_) => {
                body = Some(parsed_body);
            }
            _ => {}
        }
    }

    return Some(Request {
        protocol_version: first_line.get(2)?.to_string(),
        path: first_line.get(1)?.to_string(),
        method: Method::from_str(first_line.get(0)?).unwrap(),
        headers,
        body,
    });
}

#[derive(Deserialize, Serialize, Debug)]
struct postRouteInput<'a> {
    message: &'a str,
}

fn handle_route(mut stream: TcpStream, req: &Request) {
    let now: DateTime<Utc> = Utc::now();

    let base_headers: response::Headers = vec![
        response::Header::Date.new(&now.to_rfc2822()),
        response::Header::CacheControl.new("public, max-age=3600"),
        response::Header::Server.new("Rust baby"),
    ];

    match req.path.as_str() {
        "/test" => {
            let mut headers = base_headers;
            headers.push(response::Header::ContentType.new("text/html;charset=utf-8"));
            headers.push(response::Header::ContentLength.new("5"));

            let response = response::Message::new(&response::Status::Ok, Some("hello"), &headers);

            stream.write_response(&response);
        }
        "/post" => {
            if let Some(body) = &req.body.to_owned() {
                let as_string = String::from_utf8_lossy(body).to_string();
                // dbg!(&as_string);
                let parsed: postRouteInput = serde_json::from_str(&as_string).unwrap();
                dbg!(parsed);
            }

            let contents = fs::read_to_string("./src/pages/index.html").unwrap();
            stream
                .write_response_body(&contents, &response::Status::Ok)
                .unwrap();
        }
        "/" => {
            let contents = fs::read_to_string("./src/pages/index.html").unwrap();

            let mut headers = base_headers;
            headers.push(response::Header::ContentType.new("text/html;charset=utf-8"));
            headers.push(response::Header::ContentLength.new(&contents.len().to_string()));

            let response = response::Message::new(&response::Status::Ok, Some(&contents), &headers);

            stream.write_response(&response);
        }
        _ => {
            let contents = fs::read_to_string("./src/pages/404.html").unwrap();
            let mut headers = base_headers;
            headers.push(response::Header::ContentType.new("text/html;charset=utf-8"));
            headers.push(response::Header::ContentLength.new(&contents.len().to_string()));

            let response =
                response::Message::new(&response::Status::NotFound, Some(&contents), &headers);

            stream.write_response(&response);
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    if let Some(request) = parse_request(&mut stream) {
        handle_route(stream, &request);
    }
}

trait HttpResponses {
    fn write_response_body(&mut self, html: &str, response: &response::Status) -> Result<(), ()>;
    fn write_response(&mut self, msg: &response::Message);
}

impl HttpResponses for TcpStream {
    fn write_response_body(&mut self, body: &str, status: &response::Status) -> Result<(), ()> {
        let msg = status.response_string();
        let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", msg, body.len(), body);
        self.write_all(response.as_bytes()).unwrap();

        return Ok(());
    }

    fn write_response(&mut self, msg: &response::Message) {
        let response = msg.to_string();
        self.write_all(response.as_bytes()).unwrap();
    }
}
