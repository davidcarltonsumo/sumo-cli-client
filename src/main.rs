extern crate cookie;
extern crate getopts;
extern crate hyper;
extern crate rpassword;
extern crate rustc_serialize;
extern crate time;

mod search;
mod session;

use search::Searcher;

use getopts::{Matches, Options};
use rpassword::read_password;
use time::{get_time, strptime};

use std::env;
use std::io::{self, Write};
use std::process::exit;

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
    options.optopt("m", "minutes",
                   "Width of search in minutes (defaults to 15)", "MINUTES");
    options.optopt("f", "from",
                   "Start time in UTC; can't be used if -m is provided",
                   "START_TIME");
    options.optopt("t", "to", "End time in UTC (defaults to now)",
                   "END_TIME");
    options.optopt("x", "max-count", "Max result count (defaults to 1000)",
                   "MAX_COUNT");
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

    let (start, end) = parse_time(&matches);

    let max = opt_num_or(&matches, "x", 1000);

    if matches.free.len() != 1 {
        print_usage(&program, options);
        return;
    }

    let query = matches.free[0].clone();

    write!(io::stderr(), "Password: ").unwrap();
    io::stderr().flush().unwrap();
    let password = read_password().unwrap();

    let debug = matches.opt_present("d");

    let searcher = Searcher::new(&endpoint,
                                 &username, &password,
                                 &query,
                                 start, end,
                                 debug);

    searcher.complete_search(max);
}

fn print_usage(program: &str, options: Options) {
    let brief = format!("Usage: {} [options] -u username QUERY", program);
    print!("{}", options.usage(&brief));
}

fn parse_time(matches: &Matches) -> (i64, i64) {
    let time_format = "%Y-%m-%dT%H:%M:%S";
    let time_error =
        "Time must be in format YYYY-MM-DDTHH:MM:SS, e.g. 2015-08-20T13:54:13";

    if matches.opt_present("m") && matches.opt_present("f") {
        println!("You can specify at most one of -m and -f.");
        exit(1);
    }

    let end_time = if matches.opt_present("t") {
        match strptime(&matches.opt_str("t").unwrap(),
                       time_format) {
            Ok(t) => t.to_timespec(),
            Err(_) => {
                println!("{}", time_error);
                exit(1)
            },
        }
    } else {
        get_time()
    };
    let end = end_time.sec * 1000;

    let start = if matches.opt_present("f") {
        match strptime(&matches.opt_str("f").unwrap(),
                       time_format) {
            Ok(t) => t.to_timespec().sec * 1000,
            Err(_) => {
                println!("{}", time_error);
                exit(1)
            },
        }
    } else {
        let minutes = opt_num_or(matches, "m", 15);
        end - minutes * 60 * 1000
    };

    (start, end)
}

fn opt_num_or(matches: &Matches,
              arg: &str,
              default_value: i64) -> i64 {
    let match_opt = matches.opt_str(arg);
    match_opt.map(|s| s.parse::<i64>().unwrap_or_else(|_| {
        println!("Non-numeric argument to -{}", arg);
        exit(1)
    })).unwrap_or(default_value)
}
