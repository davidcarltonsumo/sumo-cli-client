extern crate cookie;
extern crate hyper;
extern crate rpassword;
extern crate rustc_serialize;
extern crate time;

mod session;

use session::Session;

use hyper::client::Client;
use hyper::client::response::Response;
use rpassword::read_password;
use rustc_serialize::json;
use time::get_time;

use std::env;
use std::io;
use std::io::{Read, Write};

#[derive(RustcEncodable)]
#[allow(non_snake_case)]
struct CreateRequest {
    query: String,
    from: String,
    to: String,
    timeZone: String,
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let endpoint = "https://api.sumologic.com/api/v1/search/jobs";
    let args: Vec<_> = env::args().collect();
    let ref username = args[1];
    print!("Password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    let client = Client::new();

    let mut session = Session::new(username, &password);
    let now = time::get_time();
    let end = now.sec * 1000;
    let start = end - (60 * 1000);

    let request = CreateRequest {
        query: "error".to_owned(),
        from: start.to_string(),
        to: end.to_string(),
        timeZone: "UTC".to_owned(),
    };

    let body = json::encode(&request).unwrap();
    println!("{}", body);

    let mut creation_response = client.post(endpoint)
        .headers(session.current_headers())
        .body(&body)
        .send()
        .unwrap();

    let creation_body = print_response(&mut creation_response);

    session.on_creation(&creation_response.headers, &creation_body);
    println!("New URL: {}", session.url());

    let mut status_response = client.get(&session.url())
        .headers(session.current_headers())
        .send()
        .unwrap();

    print_response(&mut status_response);

    let mut delete_response = client.delete(&session.url())
        .headers(session.current_headers())
        .send()
        .unwrap();

    print_response(&mut delete_response);
}

fn print_response(response: &mut Response) -> String {
    println!("Status: {}", response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    println!("Response: {}", response_body);
    response_body
}
