extern crate hyper;
extern crate rpassword;
extern crate time;

use hyper::client::Client;
use hyper::header::{Headers, Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rpassword::read_password;
use std::env;
use std::io;
use std::io::{Read, Write};
use time::get_time;

fn main() {
    let endpoint = "https://api.sumologic.com/api/v1/search/jobs";
    let args: Vec<_> = env::args().collect();
    let ref username = args[1];
    print!("Password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    let client = Client::new();

    let mut headers = Headers::new();
    headers.set(Authorization(Basic {
        username: username.to_owned(),
        password: Some(password.to_owned())
    }));
    headers.set(ContentType(Mime(
        TopLevel::Application, SubLevel::Json,
        vec![(Attr::Charset, Value::Utf8)]
            )));
    let now = time::get_time();
    let end = now.sec * 1000;
    let start = end - (60 * 1000);

    let ref body = format!(r#"{{"query":"error","from":"{}","to":"{}","timeZone":"UTC"}}"#,
                           start, end);
    println!("{}", body);

    let mut res = client.post(endpoint)
        .headers(headers)
        .body(body)
        .send()
        .unwrap();

    println!("Status: {}", res.status);
    let mut response_body = String::new();
    res.read_to_string(&mut response_body).unwrap();
    println!("Response: {}", response_body);
}
