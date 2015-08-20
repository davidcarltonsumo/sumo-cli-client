extern crate cookie;
extern crate getopts;
extern crate hyper;
extern crate rpassword;
extern crate rustc_serialize;
extern crate time;

mod search;
mod session;

use search::Searcher;

use getopts::Options;
use rpassword::read_password;
use time::get_time;

use std::env;
use std::io::{self, Write};

static DEFAULT_ENDPOINT: &'static str =
    "https://api.sumologic.com/api/v1/search/jobs";

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let mut options = Options::new();
    options.optopt("u", "username", "Your username", "USERNAME");
    options.optopt("e", "endpoint",
                   &format!("The full API endpoint, e.g. {}",
                            DEFAULT_ENDPOINT),
                   "ENDPOINT");
    options.optflag("d", "debug", "Print extra debugging information");
    options.optflag("h", "help", "Print this help menu");

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let matches = match options.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, options);
        return;
    }

    let endpoint = matches.opt_str("e").unwrap_or(DEFAULT_ENDPOINT.to_owned());

    if !matches.opt_present("u") {
        print_usage(&program, options);
        return;
    }
    let username = matches.opt_str("u").unwrap();

    if matches.free.len() != 1 {
        print_usage(&program, options);
        return;
    }

    let query = matches.free[0].clone();

    print!("Password: ");
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

    let now = time::get_time();
    let end = now.sec * 1000;
    let start = end - (60 * 1000);

    let debug = matches.opt_present("d");

    let searcher = Searcher::new(&endpoint,
                                 &username, &password,
                                 &query,
                                 start, end,
                                 debug);

    searcher.complete_search();
}

fn print_usage(program: &str, options: Options) {
    let brief = format!("Usage: {} [-e endpoint] -u username QUERY", program);
    print!("{}", options.usage(&brief));
}
