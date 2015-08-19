extern crate hyper;
extern crate rpassword;
extern crate time;

mod session;

use session::Session;

use hyper::client::Client;
use rpassword::read_password;
use std::env;
use std::io;
use std::io::{Read, Write};
use time::get_time;

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

    let ref body = format!(r#"{{"query":"error","from":"{}","to":"{}","timeZone":"UTC"}}"#,
                           start, end);
    println!("{}", body);

    let mut res = client.post(endpoint)
        .headers(session.current_headers())
        .body(body)
        .send()
        .unwrap();

    println!("Status: {}", res.status);
    let mut response_body = String::new();
    res.read_to_string(&mut response_body).unwrap();
    println!("Response: {}", response_body);
    session.on_response(&res.headers);
    println!("New URL: {}", session.url());
}
