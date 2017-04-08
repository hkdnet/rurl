
use std::process::exit;
use std::io::{Write, Read};
use std::net::TcpStream;
use std::time::Duration;

extern crate clap;
extern crate url;
use url::{Url, ParseError};

mod http;
use http::{Request,RequestBuilder, Response, ResponseBuilder};

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
        .args_from_usage("-c, --config=[FILE]       'Sets a custom config file'
                          -A, --user-agent=[AGENT]  'User agent'
                          -d, --data=[DATA]         'http post data'
                              -X=[METHOD]           'http method'
                              <URL>                 'Sets the url to use'")
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
            .add_header("User-Agent", opt.user_agent)
            .add_header("Accept", "*/*")
            .body(opt.data)
            .finalize();
        let _ = stream.set_read_timeout(Some(Duration::new(5, 0)));
        if let Some(response) = read_stream(stream, req) {
            println!("----------------");
            println!("{}", response);
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
    data: &'a str,
    method: &'a str,
    user_agent: &'a str,
    url: &'a str,
}

impl<'a> RurlOption<'a> {
    fn get_url(&self) -> Result<Url, ParseError> {
        Url::parse(self.url)
    }
}

fn build_option<'a>(matches: &'a clap::ArgMatches) -> RurlOption<'a> {
    let method = matches.value_of("X").unwrap_or("GET");
    let user_agent = matches.value_of("user-agent").unwrap_or("rurl/0.1.0");
    let data = matches.value_of("data").unwrap_or("");
    let url = matches.value_of("URL");
    RurlOption {
        data: data,
        method: method,
        user_agent: user_agent,
        url: url.unwrap()
    }
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