extern crate encoding;
extern crate regex;
extern crate clap;
extern crate url;
extern crate serde_json;
mod http;

use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use encoding::{Encoding, EncoderTrap};
use encoding::all::UTF_8;
use regex::Regex;
use http::{Request, Response, Operation};
use clap::{Arg, App};
use url::Url;

const PORT_SUFFIX: &str = ":8080";

fn get_resource(url: &str) -> String {
    let re = Regex::new(r"^[^/]+//[^/]+/([^\s;]+).*").unwrap();
    match re.is_match(url) {
        true => {
            let caps = re.captures(url).unwrap();
            return format!("/{}", caps.get(1).unwrap().as_str());
        }
        false => {
            return String::from("/")
        }
    }
}

fn post_resource(url: &str) -> String {
    let re = Regex::new(r"^[^/]+//[^/]+/([^\s;?]+).*").unwrap();
    match re.is_match(url) {
        true => {
            let caps = re.captures(url).unwrap();
            return format!("/{}", caps.get(1).unwrap().as_str());
        }
        false => {
            return String::from("/")
        }
    }
}

fn parse_headers(header_string: &str) -> HashMap<String, String> {
    let mut headers: HashMap<String, String> = HashMap::new();
    let re = Regex::new("([^\\s\"]+)(:\\s)([^\"]+)").unwrap();
    if re.is_match(header_string){
        let caps = re.captures_iter(header_string);
        for cap in caps {
            headers.insert(format!("{}", cap.get(1).unwrap().as_str()),
                          format!("{}", cap.get(3).unwrap().as_str()));
        }
    }
    return headers
}

fn run_client(url: &str, operation: Operation, resource: String,
              headers: HashMap<String, String>, body: String, verbose: bool, outfile: String){
    if url.contains("https") {
        panic!("Not compatible with secure HTTP (https).");
    }
    let parsed_url = Url::parse(url).unwrap();
    let host = parsed_url.host_str().unwrap();
    let mut address = format!("{}{}", host, PORT_SUFFIX).to_socket_addrs().unwrap();
    match TcpStream::connect(&address.next().unwrap()) {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_millis(1000)));
            let request = Request::new(&host, &operation,
                                       &resource, &headers, &body);
            let content_bytes = UTF_8.encode(&request.to_string(), EncoderTrap::Strict).unwrap();
            let _req = stream.write(&content_bytes).unwrap();
            let mut buf = String::new();
            let _resp = match stream.read_to_string(&mut buf) {
                Ok(resp) => resp,
                Err(_) => {
                    0
                }
            };
            let response = Response::from_str(&buf);
            if !outfile.is_empty() {
            	let mut file = OpenOptions::new().append(true).create(true).open(outfile.to_string()).unwrap();
            	if verbose {
            		file.write_all(response.status_line.as_bytes()).unwrap();
            		for (k, v) in response.headers.to_owned(){
            			file.write_all(format!("{}: {}\r\n", k, v).as_bytes()).unwrap();
            		}
            	}
                file.write_all(response.body.as_bytes()).unwrap()
            }
            else {
                if verbose {
                    println!("{}", response.status_line);
                    for (k, v) in response.headers.to_owned() {
                        println!("{}: {}", k, v);
                    }
                }
                println!("{}", response.body);
            }
            if response.status_line.contains("302") {
                let new_url = response.headers.get("Location").unwrap();
                println!("Redirect to {}", new_url);
                run_client(new_url, operation, resource, headers, body, verbose, outfile);
            }
        },
        _ => {
            println!("Failed to connect.");
        }
    }
}

fn main() {
    let args = App::new("httpc: a Rust implementation of curl")
                            .version("0.1")
                            .arg(Arg::with_name("operation")
                                .help("Operation to be executed.")
                                .required(true)
                                .possible_values(&["get", "post"])
                                .index(1))
                            .arg(Arg::with_name("verbose")
                                .short("v")
                                .help("Prints all details on a resource, such as protocol, status, and headers."))
                            .arg(Arg::with_name("header")
                                .short("h")
                                .help("Associates headers to HTTP Request with the format 'key:value'")
                                .takes_value(true)
                                .value_name("HEADERS"))
                            .arg(Arg::with_name("inline")
                                .long("d")
                                .help("Allows for passing request body from command line parameter.")
                                .takes_value(true)
                                .value_name("INLINE"))
                            .arg(Arg::with_name("file")
                                .long("f")
                                .help("Pass filename and contents of file will be sent as request body.")
                                .takes_value(true)
                                .value_name("FILE"))
                            .arg(Arg::with_name("url")
                                .help("The url to the resource you wish to request.")
                                .index(2)
                                .required(true)
                                .value_name("URL"))
                            .arg(Arg::with_name("output")
                                .short("o")
                                .takes_value(true)
                                .value_name("FILE")).get_matches();
    let url = args.value_of("url").unwrap();
    let operation = match args.value_of("operation").unwrap() {
        "get" => Operation::Get,
        "post" => Operation::Post,
        _ => panic!()
    };
    let verbose = args.is_present("verbose");
    let headers = match args.is_present("header") {
        true => parse_headers(args.value_of("header").unwrap()),
        false => HashMap::new()
    };
    let inline = args.is_present("inline");
    let file = args.is_present("file");
    let outfile = match args.is_present("output") {
        true => String::from(args.value_of("output").unwrap()),
        false => String::from("")
    };
    let resource: String = match operation {
        Operation::Get => get_resource(url),
        Operation::Post => post_resource(url)
    };
    let body = match operation {
        Operation::Get => String::from(""),
        Operation::Post => {
            if inline {
                String::from(args.value_of("inline").unwrap())
            }
            else if file {
                let path = Path::new(args.value_of("file").unwrap());

                let mut file = File::open(&path).unwrap();

                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents).unwrap();
                file_contents
            }
            else {
                panic!("Post request without inline or file.")
            }
        }
    };
    run_client(url, operation, resource, headers, body, verbose, outfile);
}