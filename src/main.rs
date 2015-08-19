extern crate hyper;
extern crate rpassword;

use hyper::client::Client;
use hyper::header::{Headers, Authorization, Basic};
use rpassword::read_password;
use std::env;
use std::io;
use std::io::Write;

fn main() {
    let endpoint = "https://api.sumologic.com/api/v1/search/jobs";
    let args: Vec<_> = env::args().collect();
    let ref username = args[1];
    print!("Password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    let client = Client::new();

    let mut headers = Headers::new();
    headers.set(
        Authorization(
            Basic {
                username: username.to_owned(),
                password: Some(password.to_owned())
            }
            )
            );

    let res = client.post(endpoint)
        .headers(headers)
        .send()
        .unwrap();

    println!("{}", res.status);
}
