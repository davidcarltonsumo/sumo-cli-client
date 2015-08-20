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
}

impl Searcher {
    pub fn new(endpoint: &str,
               username: &str,
               password: &str,
               query: &str,
               start: i64,
               end: i64) -> Searcher {
        let mut searcher = Searcher {
            client: Client::new(),
            session: Session::new(username, password)
        };

        let request = CreateRequest {
            query: query.to_owned(),
            from: start.to_string(),
            to: end.to_string(),
            timeZone: "UTC".to_owned(),
        };

        let body = json::encode(&request).unwrap();
        println!("{}", body);

        let mut creation_response = searcher.client.post(endpoint)
            .headers(searcher.session.current_headers())
            .body(&body)
            .send()
            .unwrap();

        let creation_body = print_response(&mut creation_response);

        searcher.session.on_creation(&creation_response.headers,
                                     &creation_body);
        println!("New URL: {}", searcher.session.url());

        searcher
    }

    pub fn complete_search(&self) {
        let mut status_response = self.client.get(&self.session.url())
            .headers(self.session.current_headers())
            .send()
            .unwrap();

        print_response(&mut status_response);

        let mut delete_response = self.client.delete(&self.session.url())
            .headers(self.session.current_headers())
            .send()
            .unwrap();

        print_response(&mut delete_response);
    }
}

fn print_response(response: &mut Response) -> String {
    println!("Status: {}", response.status);
    let mut response_body = String::new();
    response.read_to_string(&mut response_body).unwrap();
    println!("Response: {}", response_body);
    response_body
}
