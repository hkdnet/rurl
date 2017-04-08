use std::fmt;

#[derive(Debug)]
pub struct Request {
    method: String,
    path: String,
    headers: Vec<HttpHeader>,
    body: String
}
impl Request {
    fn get_headers(&self) -> Vec<HttpHeader> {
        self.headers.to_vec()
    }
    pub fn to_string(&self) -> String {
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
pub struct RequestBuilder {
    method: String,
    path: String,
    headers: Vec<HttpHeader>,
    body: String
}
impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        let vec = Vec::<HttpHeader>::with_capacity(4);
        RequestBuilder { headers: vec, method: "".to_string(), path: "".to_string(), body: "".to_string() }
    }
    pub fn method(&self, method: &str) -> RequestBuilder {
        RequestBuilder { headers: self.headers.to_vec(), method: method.to_string(), path: self.path.to_string(), body: self.body.to_string()}
    }
    pub fn path(&self, path: &str) -> RequestBuilder {
        RequestBuilder { headers: self.headers.to_vec(), method: self.method.to_string(), path: path.to_string(), body: self.body.to_string()}
    }
    pub fn add_header(&self, key: &str, value: &str) -> RequestBuilder {
        let header = HttpHeader::new(key, value);
        let mut new_headers = self.headers.to_vec();
        new_headers.push(header);
        RequestBuilder { headers: new_headers , method: self.method.to_string(), path: self.path.to_string(), body: self.body.to_string()}
    }
    pub fn body(&self, body: &str) -> RequestBuilder {
        RequestBuilder { headers: self.headers.to_vec(), method: self.method.to_string(), path: self.path.to_string(), body: body.to_string() }
    }
    pub fn finalize(&self) -> Request {
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

#[derive(Debug)]
pub struct Response {
    protocol: String,
    status: i32,
    message: String,
    headers: Vec<HttpHeader>,
    body: String
}
impl Response {
    pub fn to_string(&self) -> String {
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
impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug)]
pub struct ResponseBuilder {
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
    pub fn parse_response(response: &str) -> Option<Response> {
        println!("***********************");
        let builder = ResponseBuilder::new();
        let tmp = response.splitn(2, "\r\n\r\n").collect::<Vec<&str>>();
        println!("{}", tmp[0]);
        println!("***********************");
        println!("{}", tmp[1]);
        println!("***********************");
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