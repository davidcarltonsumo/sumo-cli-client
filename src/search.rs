use session::Session;

use hyper::client::Client;
use hyper::client::response::Response;
use rustc_serialize::json;

use std::io::Read;

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
        let mut status_response = self.client.get(&self.session.url())
            .headers(self.session.current_headers())
            .send()
            .unwrap();

        self.consume_response(&mut status_response);

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
