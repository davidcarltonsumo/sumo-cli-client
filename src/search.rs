use session::Session;

use hyper::client::Client;
use hyper::client::response::Response;
use rustc_serialize::json::{self, Json};

use std::cmp::min;
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

#[derive(RustcDecodable)]
#[allow(non_snake_case)]
struct StatusResult {
    state: String,
    messageCount: i64,
    recordCount: i64,
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
        let mut last_status_result_opt: Option<StatusResult>;

        loop {
            thread::sleep_ms(POLL_INTERVAL);

            let mut status_response = self.client.get(&self.session.url())
                .headers(self.session.current_headers())
                .send()
                .unwrap();

            let body = self.consume_response(&mut status_response);
            last_status_result_opt = json::decode(&body).ok();
            let state = last_status_result_opt.as_ref()
                .map(|s| s.state.to_owned())
                .unwrap_or("UNKNOWN".to_owned());

            match state.as_ref() { 
                "NOT STARTED" | "GATHERING RESULTS" => continue,
                _ => break,
            }
        }

        match last_status_result_opt {
            Some(status_result) => {
                if status_result.state == "DONE GATHERING RESULTS" {
                    self.fetch_results(status_result);
                } else {
                    println!("Query did not finish correctly, state={}",
                             status_result.state);
                }
            }
            None => {
                println!("Error getting status!")
            }
        }

        let mut delete_response = self.client.delete(&self.session.url())
            .headers(self.session.current_headers())
            .send()
            .unwrap();

        self.consume_response(&mut delete_response);
    }

    fn fetch_results(&self, status_result: StatusResult) {
        // This isn't a good heuristic - I filed SUMO-47206 for that.
        let method = if status_result.recordCount > 0 {
            "records"
        } else {
            "messages"
        };
        let count = if status_result.recordCount > 0 {
            status_result.recordCount
        } else {
            status_result.messageCount
        };

        let mut offset = 0;
        while offset < count {
            let url = format!("{}/{}?offset={}&limit={}",
                              self.session.url(),
                              method,
                              offset,
                              min(count - offset, 10000));

            let mut fetch_response = self.client.get(&url)
                .headers(self.session.current_headers())
                .send()
                .unwrap();
            let fetch_body = self.consume_response(&mut fetch_response);

            for row in rows_from(&fetch_body, method).iter() {
                println!("{}", row);
            }

            offset += 10000;
        }
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

fn rows_from(fetch_body: &str,
             method: &str) -> Vec<Json> {
    // FIXME (2015-08-15, carlton): I'd like to do the typed JSON
    // decoding here, but it doesn't let me decode partway and then
    // return Json.
    Json::from_str(fetch_body).unwrap()
        .as_object().unwrap()
        .get(method).unwrap()
        .as_array().unwrap().iter()
        .map(|record|
             record.as_object().unwrap()
             .get("map").unwrap()
             .to_owned())
        .collect()
}
