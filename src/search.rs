use session::Session;

use hyper::client::Client;
use hyper::client::response::Response;
use rustc_serialize::json;

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
        let url: String = if status_result.recordCount > 0 {
            format!("{}/records?offset=0&limit={}",
                    self.session.url(),
                    min(status_result.recordCount, 10000))
        } else {
            format!("{}/messages?offset=0&limit={}",
                    self.session.url(),
                    min(status_result.messageCount, 10000))
        };
        let mut fetch_response = self.client.get(&url)
            .headers(self.session.current_headers())
            .send()
            .unwrap();

        println!("{}", self.consume_response(&mut fetch_response));
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
