use session::Session;

use hyper::client::Client;
use hyper::client::response::Response;
use rustc_serialize::json::{self, Json};

use std::io::Read;
use std::thread;

#[derive(RustcEncodable)]
#[allow(non_snake_case)]
struct CreateRequest {
    query: String,
    from: String,
    to: String,
    timeZone: String,
}

pub struct Searcher {
    client: Client,
    session: Session,
    debug: bool,
}

static POLL_INTERVAL: u32 = 300; // milliseconds

impl Searcher {
    pub fn new(endpoint: &str,
               username: &str,
               password: &str,
               query: &str,
               start: i64,
               end: i64,
               debug: bool) -> Searcher {
        let mut searcher = Searcher {
            client: Client::new(),
            session: Session::new(username, password, debug),
            debug: debug,
        };

        let request = CreateRequest {
            query: query.to_owned(),
            from: start.to_string(),
            to: end.to_string(),
            timeZone: "UTC".to_owned(),
        };

        let body = json::encode(&request).unwrap();
        if searcher.debug {
            println!("{}", body);
        }

        let mut creation_response = searcher.client.post(endpoint)
            .headers(searcher.session.current_headers())
            .body(&body)
            .send()
            .unwrap();

        let creation_body = searcher.consume_response(&mut creation_response);

        searcher.session.on_creation(&creation_response.headers,
                                     &creation_body);
        if searcher.debug {
            println!("New URL: {}", searcher.session.url());
        }

        searcher
    }

    pub fn complete_search(&self) {
        let mut last_status_response_opt: Option<Json>;

        loop {
            thread::sleep_ms(POLL_INTERVAL);

            let mut status_response = self.client.get(&self.session.url())
                .headers(self.session.current_headers())
                .send()
                .unwrap();

            let body = self.consume_response(&mut status_response);
            last_status_response_opt = Json::from_str(&body).ok();
            let status = last_status_response_opt.as_ref()
                .and_then(|o| o.as_object())
                .and_then(|o| o.get("state"))
                .and_then(|o| o.as_string())
                .unwrap_or("UNKNOWN");

            match status {
                "NOT STARTED" | "GATHERING RESULTS" => continue,
                _ => break,
            }
        }

        for response_obj in (last_status_response_opt.as_ref()
                             .and_then(|o| o.as_object()).iter()) {
            println!("messageCount: {}",
                     response_obj.get("messageCount")
                     .and_then(|o| o.as_i64())
                     .map(|o| o.to_string())
                     .unwrap_or("unknown".to_owned()));
            println!("recordCount: {}",
                     response_obj.get("recordCount")
                     .and_then(|o| o.as_i64())
                     .map(|o| o.to_string())
                     .unwrap_or("unknown".to_owned()));
        }

        let mut delete_response = self.client.delete(&self.session.url())
            .headers(self.session.current_headers())
            .send()
            .unwrap();

        self.consume_response(&mut delete_response);
    }

    fn consume_response(&self, response: &mut Response) -> String {
        if self.debug {
            println!("Status: {}", response.status);
        }
        let mut response_body = String::new();
        response.read_to_string(&mut response_body).unwrap();
        if self.debug {
            println!("Response: {}", response_body);
        }
        response_body
    }
}
