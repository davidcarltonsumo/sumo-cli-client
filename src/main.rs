extern crate cookie;
extern crate hyper;
extern crate rpassword;
extern crate rustc_serialize;
extern crate time;

mod search;
mod session;

use search::Searcher;

use rpassword::read_password;
use time::get_time;

use std::env;
use std::io::{self, Write};

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let endpoint = "https://api.sumologic.com/api/v1/search/jobs";

    let args: Vec<_> = env::args().collect();
    let ref username = args[1];

    print!("Password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    let now = time::get_time();
    let end = now.sec * 1000;
    let start = end - (60 * 1000);

    let searcher = Searcher::new(endpoint, username, &password,
                                 "error", start, end);

    searcher.complete_search();
}
