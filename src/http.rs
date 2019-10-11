use std::collections::HashMap;
use regex::Regex;

const APPLICATION: &str = "r_url";
const VERSION: &str = "0.1";

pub enum Operation {
    Get,
    Post
}

pub struct Request {
    pub uri: String,
    pub request_line: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Request {
    pub fn new(host: &str, operation: &Operation, resource: &String, head: &HashMap<String, String>, b: &String) -> Request{
        let op = match operation {
            Operation::Get => String::from("GET"),
            Operation::Post => String::from("POST")
        };
        let request_line: String = format!("{} {} HTTP/1.1", op, resource);
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert(String::from("Host"), host.to_string());
        headers.insert(String::from("User-Agent"), format!("{}/{}", APPLICATION, VERSION));
        headers.insert(String::from("Accept-Language"), String::from("en-us"));
        headers.insert(String::from("Accept-Encoding"), String::from("utf-8"));
        headers.insert(String::from("Connection"), String::from("keep-alive"));
        let body = b.to_string();
        if !body.is_empty() {
            headers.insert(String::from("Content-Type"), String::from("application/json"));
            headers.insert(String::from("Content-Length"), format!("{}", body.chars().count()));
        }
        if !head.is_empty() {
            for (key, val) in head {
                headers.insert(key.to_string(), val.to_string());
            }
        }
        Request {
            uri: String::from(host),
            request_line,
            headers,
            body
        }
    }

    pub fn to_string(&self) -> String {
        let mut output: String = String::new();
        output.push_str(&format!("{}\r\n", self.request_line));
        for (k, v) in self.headers.iter() {
            output.push_str(&format!("{}: {}\r\n", k, v))
        }
        output.push_str("\r\n");
        output.push_str(&format!("{}\r\n", self.body));
        return output;
    }
}

pub struct Response {
    pub status_line: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Response {
    pub fn from_str(raw_text: &String) -> Response{
        let mut lines = raw_text.lines();
        let status_line = match lines.next() {
            Some(line) => String::from(line),
            None => String::from("")
        };
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = String::new();

        let re = Regex::new("([^\\s\"]+)(:\\s)([^\"]+)").unwrap();
        for line in lines{
            if re.is_match(&line){
                let caps = re.captures(&line).unwrap();
                headers.insert(format!("{}", caps.get(1).unwrap().as_str()),
                               format!("{}", caps.get(3).unwrap().as_str()));
            }
            else {
                body.push_str(&format!("{}\n", line));
            }
        }

        Response{
            status_line,
            headers,
            body
        }
    }

    pub fn to_string(&self) -> String {
        let mut output: String = String::new();
        output.push_str(&format!("{}\r\n", self.status_line));
        for (k, v) in self.headers.iter() {
            output.push_str(&format!("{}: {}\r\n", k, v))
        }
        output.push_str("\r\n");
        output.push_str(&format!("{}\r\n", self.body));
        return output;
    }
}