use cookie::Cookie;

#[derive(Debug, Clone)]
pub struct CookieJar {
    cookies: Vec<Cookie<'static>>,
}

impl CookieJar {
    pub fn new() -> CookieJar {
        CookieJar {
            cookies: Vec::new(),
        }
    }

    pub fn add(&mut self, cookie: String) {
        let cookie = Cookie::parse(cookie).unwrap();
        if cookie.name() == "clid" || cookie.name() == "asid" {
            return;
        }
        // Don't add cookie if it already exists and not expired
        if self.cookies.iter().any(|c| c.name() == cookie.name()) {
            return;
        }
        self.cookies.push(cookie);
    }

    pub fn create_cookie_header(&self) -> String {
        let mut cookie_header = String::new();
        for cookie in &self.cookies {
            cookie_header.push_str(&cookie.name());
            cookie_header.push_str("=");
            cookie_header.push_str(&cookie.value());
            cookie_header.push_str("; ");
        }
        cookie_header.pop();
        cookie_header.pop();
        cookie_header
    }
}
