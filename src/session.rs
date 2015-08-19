extern crate hyper;

use hyper::header::{Headers, Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

pub struct Session {
    pub headers: Headers,
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
            headers: headers
        }
    }
}
