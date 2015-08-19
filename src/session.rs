extern crate hyper;

use hyper::header::*;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

pub struct Session {
    headers: Headers,
    url_opt: Option<String>,
}

impl Session {
    pub fn new(username: &str, password: &str) -> Session {
        let mut headers = Headers::new();
        headers.set(Authorization(Basic {
            username: username.to_owned(),
            password: Some(password.to_owned())
        }));
        headers.set(ContentType(Mime(
            TopLevel::Application, SubLevel::Json,
            vec![(Attr::Charset, Value::Utf8)]
                )));

        Session {
            headers: headers,
            url_opt: None,
        }
    }

    pub fn current_headers(&self) -> Headers {
        self.headers.clone()
    }

    pub fn url(&self) -> String {
        self.url_opt.as_ref().unwrap().to_owned()
    }

    pub fn on_response(&mut self, response_headers: &Headers) {
        for header in response_headers.iter() {
            println!("{}", header);
        }
        self.url_opt = response_headers.iter().filter_map(
            |header| if header.is::<Location>() {
                header.value::<Location>()
            } else {
                None
            }).next().map(|location| (**location).to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::*;
    use hyper::mime::{Mime, TopLevel, SubLevel};

    #[test]
    fn it_should_extract_the_url_from_the_response() {
        let mut session = Session::new("username", "password");

        let mut response_headers = Headers::new();

        response_headers.set(CacheControl(vec![CacheDirective::NoCache]));
        response_headers.set(Location("http://foo/bar".to_owned()));
        response_headers.set(ContentType(Mime(
            TopLevel::Application, SubLevel::Json, vec![])));

        session.on_response(&response_headers);

        assert_eq!(session.url(), "http://foo/bar");
    }
}
