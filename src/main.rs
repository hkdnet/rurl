
use std::process::exit;
use std::io::{Write, Read};
use std::net::TcpStream;
use std::vec::Vec;
use std::time::Duration;

extern crate clap;
extern crate url;
use url::{Url, ParseError};

fn main() {
    let status = run();
    exit(status);
}

macro_rules! println_error {
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
}

fn run() -> i32 {
    let matches = clap::App::new("rurl")
        .version("0.1.0")
        .author("hkdnet")
        .about("yet another curl")
        .args_from_usage("-c, --config=[FILE] 'Sets a custom config file'
                              -X=[METHOD]          'http method'
                              <URL>              'Sets the url to use'")
        .get_matches();
    let opt = build_option(&matches);
    let url_result = opt.get_url();
    if url_result.is_err() {
        println_error!("Invalid url: {}", opt.url);
        return 1;
    }
    let url = url_result.unwrap();
    // 以下はもうurlはまともになってると思う。たぶん。きっとだいじょうぶ
    let host = url.host().unwrap();
    let port = url.port().unwrap_or(default_port_for(url.scheme()));
    let host_str = format!("{}:{}", host, port);
    if let Ok(stream) = TcpStream::connect(host_str) {
        let req = RequestBuilder::new()
            .method(opt.method)
            .path(url.path())
            .add_header("Host", format!("{}:{}", host, port).as_str())
            .add_header("User-Agent", "rurl/1.0")
            .add_header("Accept", "*/*")
            .body("")
            .finalize();
        let _ = stream.set_read_timeout(Some(Duration::new(5, 0)));
        if let Some(response) = read_stream(stream, req) {
            println!("----------------");
            println!("{}", response.to_string());
        } else {
            println_error!("cannot build response...");
        }
    } else {
        println!("Couldn't connect to server...");
    }
    0
}

fn default_port_for(scheme: &str) -> u16 {
    match scheme {
        "https" => 443,
        _ => 80,
    }
}

#[derive(Debug)]
struct RurlOption<'a> {
    url: &'a str,
    method: &'a str,
}

impl<'a> RurlOption<'a> {
    fn get_url(&self) -> Result<Url, ParseError> {
        Url::parse(self.url)
    }
}

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    headers: Vec<HttpHeader>,
    body: String
}
impl Request {
    fn get_headers(&self) -> Vec<HttpHeader> {
        self.headers.to_vec()
    }
    fn to_string(&self) -> String {
        let mut vec = Vec::<String>::with_capacity(self.headers.len() + 3);
        vec.push(format!("{} {} HTTP/1.1", self.method, self.path));
        for header in self.get_headers() {
            vec.push(header.to_string());
        }
        vec.push("".to_string());
        vec.push(self.body.to_string());
        vec.join("\r\n")
    }
}
#[derive(Debug)]
struct RequestBuilder {
    method: String,
    path: String,
    headers: Vec<HttpHeader>,
    body: String
}
impl RequestBuilder {
    fn new() -> RequestBuilder {
        let vec = Vec::<HttpHeader>::with_capacity(4);
        RequestBuilder { headers: vec, method: "".to_string(), path: "".to_string(), body: "".to_string() }
    }
    fn method(&self, method: &str) -> RequestBuilder {
        RequestBuilder { headers: self.headers.to_vec(), method: method.to_string(), path: self.path.to_string(), body: self.body.to_string()}
    }
    fn path(&self, path: &str) -> RequestBuilder {
        RequestBuilder { headers: self.headers.to_vec(), method: self.method.to_string(), path: path.to_string(), body: self.body.to_string()}
    }
    fn add_header(&self, key: &str, value: &str) -> RequestBuilder {
        let header = HttpHeader::new(key, value);
        let mut new_headers = self.headers.to_vec();
        new_headers.push(header);
        RequestBuilder { headers: new_headers , method: self.method.to_string(), path: self.path.to_string(), body: self.body.to_string()}
    }
    fn body(&self, body: &str) -> RequestBuilder {
        RequestBuilder { headers: self.headers.to_vec(), method: self.method.to_string(), path: self.path.to_string(), body: body.to_string() }
    }
    fn finalize(&self) -> Request {
        Request { headers: self.headers.to_vec(), method: self.method.to_string(), path: self.path.to_string(), body: self.body.to_string() }
    }
}

#[derive(Debug, Clone)]
struct HttpHeader {
    key: String,
    value: String
}
impl HttpHeader {
    fn new(key: &str, value: &str)-> HttpHeader {
        HttpHeader { key: key.to_string(), value: value.to_string()}
    }
    fn to_string(&self) -> String {
        format!("{}: {}", self.key, self.value)
    }
}

fn build_option<'a>(matches: &'a clap::ArgMatches) -> RurlOption<'a> {
    let method = matches.value_of("X").unwrap_or("GET");
    let url = matches.value_of("URL");
    RurlOption { url: url.unwrap(), method: method }
}

fn read_stream(mut stream: std::net::TcpStream, request: Request)-> Option<Response> {
    let mut buf = [0u8; 1000];
    let req_str = request.to_string();
    println!("{}", req_str);
    let write_result = stream.write(req_str.as_bytes());
    if let Err(err) = write_result {
        println_error!("{}", err);
        return None
    }
    let read_result = stream.read(&mut buf[..]);
    if let Err(err) = read_result {
        println_error!("{}", err);
        return None
    }
    let s = std::str::from_utf8(&buf);
    let raw_response = s.unwrap();
    ResponseBuilder::parse_response(raw_response)
}

#[derive(Debug)]
struct Response {
    protocol: String,
    status: i32,
    message: String,
    headers: Vec<HttpHeader>,
    body: String
}
impl Response {
    fn to_string(&self) -> String {
        let mut vec = Vec::<String>::with_capacity(self.headers.len() + 3);
        vec.push(format!("{} {} {}", self.protocol, self.status, self.message));
        for header in self.headers.to_vec() {
            vec.push(header.to_string());
        }
        vec.push("".to_string());
        vec.push(self.body.to_string());
        vec.join("\n")
    }
}

#[derive(Debug)]
struct ResponseBuilder {
    protocol: String,
    status: i32,
    message: String,
    headers: Vec<HttpHeader>,
    body: String
}

impl ResponseBuilder {
    fn new() -> ResponseBuilder {
        let vec = Vec::<HttpHeader>::with_capacity(4);
        ResponseBuilder { protocol: "".to_string(), status: 0, message: "".to_string(), headers: vec, body: "".to_string() }
    }
    fn parse_response(response: &str) -> Option<Response> {
        let builder = ResponseBuilder::new();
        let tmp = response.splitn(2, "\r\n\r\n").collect::<Vec<&str>>();
        builder.body(tmp[1]);
        let lines = tmp[0].split("\r\n").collect::<Vec<&str>>();
        let l = lines[0];
        let tl = l.split(" ").collect::<Vec<&str>>();
        builder
            .protocol(tl[0])
            .status(tl[1])
            .message(tl[2]);
        for line in lines[1..].iter() {
            let t = line.splitn(2, ": ").collect::<Vec<&str>>();
            builder.add_header(t[0], t[1]);
        }
        let result = builder.finalize();
        Some(result)
    }
    fn protocol(&self, protocol: &str)-> ResponseBuilder {
        ResponseBuilder{ 
            protocol: protocol.to_string(), status: self.status, message: self.message.to_string(),
            headers: self.headers.to_vec(), body: self.body.to_string() 
        }
    }
    fn status(&self, status: &str)-> ResponseBuilder {
        ResponseBuilder{ 
            protocol: self.protocol.to_string(), status: status.parse::<i32>().unwrap(), message: self.message.to_string(),
            headers: self.headers.to_vec(), body: self.body.to_string() 
        }
    }
    fn message(&self, message: &str)-> ResponseBuilder {
        ResponseBuilder{ 
            protocol: self.protocol.to_string(), status: self.status, message: message.to_string(),
            headers: self.headers.to_vec(), body: self.body.to_string() 
        }
    }
    fn add_header(&self, key: &str, value: &str) -> ResponseBuilder {
        let header = HttpHeader{ key: key.to_string(), value: value.to_string() };
        let mut vec = self.headers.to_vec();
        vec.push(header);
        ResponseBuilder {
            protocol: self.protocol.to_string(), status: self.status, message: self.message.to_string(),
            headers: vec, body: self.body.to_string()
        }
    }
    fn body(&self, body: &str)-> ResponseBuilder {
        ResponseBuilder{ 
            protocol: self.protocol.to_string(), status: self.status, message: self.message.to_string(),
            headers: self.headers.to_vec(), body: body.to_string() 
        }
    }
    fn finalize(&self)-> Response {
        Response{ 
            protocol: self.protocol.to_string(), status: self.status, message: self.message.to_string(),
            headers: self.headers.to_vec(), body: self.body.to_string() 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut headers = Vec::<HttpHeader>::new();
        headers.push(HttpHeader{key: "KEY".to_string(), value: "value".to_string()});
        let res = Response {
            headers: headers,
            protocol: "HTTP/1.1".to_string(),
            status: 200,
            message: "OK".to_string(),
            body: "body".to_string()
        };
        let expected = "HTTP/1.1 200 OK
KEY: value

body";
        assert_eq!(expected, res.to_string());

    }
}

// > GET /status HTTP/1.1
// > Host: localhost:12030
// > User-Agent: curl/7.43.0
// > Accept: */*
// >
// < HTTP/1.1 200 OK
// < X-Frame-Options: SAMEORIGIN
// < X-XSS-Protection: 1; mode=block
// < X-Content-Type-Options: nosniff
// < Content-Type: application/json; charset=utf-8
// < ETag: W/"7cd85494eb375cc958155aca095fd0ba"
// < Cache-Control: max-age=0, private, must-revalidate
// < X-Request-Id: 01490cdd-b81d-4b32-a877-5ccdcf4971cc
// < X-Runtime: 0.005148
// < Vary: Origin
// < Transfer-Encoding: chunked
// <
// * Connection #0 to host localhost left intact