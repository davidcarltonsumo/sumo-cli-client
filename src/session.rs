use cookie::CookieJar;
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

        self.url_opt = header_of_type::<Location>(response_headers)
            .map(|location| normalize_url(&***location));

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

// SUMO-47175; the URL in the body of the response is correct, so once I'm
// parsing that, I can get rid of this.
fn normalize_url(url: &str) -> String {
    let components: Vec<_> = url.splitn(2, ":").collect();
    if components[0] == "http" {
        "https:".to_owned() + components[1]
    } else {
        url.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cookie::Cookie as CookiePair;
    use hyper::header::*;
    use hyper::mime::{Mime, TopLevel, SubLevel};
    use std::collections::HashMap;

    #[test]
    fn it_should_extract_the_url_from_the_response() {
        let mut session = Session::new("username", "password");

        let mut response_headers = Headers::new();

        response_headers.set(CacheControl(vec![CacheDirective::NoCache]));
        response_headers.set(Location("https://foo/bar".to_owned()));
        response_headers.set(ContentType(Mime(
            TopLevel::Application, SubLevel::Json, vec![])));

        session.on_response(&response_headers);

        assert_eq!(session.url(), "https://foo/bar");
    }

    // SUMO-47175; the URL in the body of the response is correct, so once I'm
    // parsing that, I can get rid of this.
    #[test]
    fn it_should_make_sure_the_url_is_https() {
        let mut session = Session::new("username", "password");

        let mut response_headers = Headers::new();

        response_headers.set(CacheControl(vec![CacheDirective::NoCache]));
        response_headers.set(Location("http://foo/bar".to_owned()));
        response_headers.set(ContentType(Mime(
            TopLevel::Application, SubLevel::Json, vec![])));

        session.on_response(&response_headers);

        assert_eq!(session.url(), "https://foo/bar");
    }

    #[test]
    fn it_should_send_back_cookies_from_the_create_request() {
        let mut session = Session::new("username", "password");

        let mut response_headers = Headers::new();

        response_headers.set(Location("http://foo/bar".to_owned()));
        response_headers.set(SetCookie(vec![
            CookiePair::new("Key1".to_owned(), "val1".to_owned()),
            CookiePair::new("Key2".to_owned(), "val2".to_owned()),
            ]));

        session.on_response(&response_headers);

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
