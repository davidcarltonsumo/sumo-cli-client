use cookie::CookieJar;
use hyper::header::*;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json::Json;

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

    pub fn on_creation(&mut self,
                       response_headers: &Headers,
                       response_body: &str) {
        for header in response_headers.iter() {
            println!("{}", header);
        }

        // SUMO-47175: Don't trust the Location header, grab the
        // url from the body instead.
        let body_json_result = Json::from_str(response_body);
        self.url_opt = body_json_result.ok().as_ref()
            .and_then(|o| o.as_object())
            .and_then(|o| o.get("link"))
            .and_then(|o| o.as_object())
            .and_then(|o| o.get("href"))
            .and_then(|o| o.as_string())
            .map(|s| s.to_owned());

        let cookie_header_opt = header_of_type::<SetCookie>(response_headers);
        let mut cookie_jar = CookieJar::new(
            b"2301b982cd730fe192730192873af394");
        for cookie in cookie_header_opt.iter() {
            cookie.apply_to_cookie_jar(&mut cookie_jar);
        }

        self.headers.set(Cookie::from_cookie_jar(&mut cookie_jar));
    }
}

fn header_of_type<H: Header + HeaderFormat>(headers: &Headers) -> Option<&H> {
    headers.iter().filter_map(
        |header| if header.is::<H>() {
            header.value::<H>()
        } else {
            None
        }
        ).next()
}

#[cfg(test)]
mod tests {
    use super::*;

    use cookie::Cookie as CookiePair;
    use hyper::header::*;
    use std::collections::HashMap;

    static SIMPLE_BODY: &'static str = r#"{"id":"012345689ABCDEF00","link":{"rel":"self","href":"https://foo/bar/012345689ABCDEF00"}}"#;

    #[test]
    fn it_should_extract_the_url_from_the_response() {
        let mut session = Session::new("username", "password");

        let response_headers = Headers::new();

        session.on_creation(&response_headers, SIMPLE_BODY);

        assert_eq!(session.url(), "https://foo/bar/012345689ABCDEF00");
    }

    #[test]
    fn it_should_send_back_cookies_from_the_create_request() {
        let mut session = Session::new("username", "password");

        let mut response_headers = Headers::new();

        response_headers.set(Location(
            "https://foo/bar/012345689ABCDEF00".to_owned()));
        response_headers.set(SetCookie(vec![
            CookiePair::new("Key1".to_owned(), "val1".to_owned()),
            CookiePair::new("Key2".to_owned(), "val2".to_owned()),
            ]));

        session.on_creation(&response_headers, SIMPLE_BODY);

        let updated_headers = session.current_headers();
        let cookie_header =
            super::header_of_type::<Cookie>(&updated_headers).unwrap();

        let mut cookie_map: HashMap<String, String> = HashMap::new();
        for cookie in cookie_header.iter() {
            cookie_map.insert(cookie.name.to_owned(), cookie.value.to_owned());
        }

        assert_eq!(cookie_map.len(), 2);
        assert_eq!(cookie_map.get("Key1").unwrap(), "val1");
        assert_eq!(cookie_map.get("Key2").unwrap(), "val2");
    }
}
