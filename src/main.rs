
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
        read_stream(stream, req);
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

fn read_stream(mut stream: std::net::TcpStream, request: Request) {
    let mut buf = [0u8; 1000];
    let req_str = request.to_string();
    println!("{}", req_str);
    let write_result = stream.write(req_str.as_bytes());
    if let Err(err) = write_result {
        println_error!("{}", err);
        return
    }
    let read_result = stream.read(&mut buf[..]);
    if let Err(err) = read_result {
        println_error!("{}", err);
        return
    }
    let s = std::str::from_utf8(&buf);
    println!("{}", s.unwrap());
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