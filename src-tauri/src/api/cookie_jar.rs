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
        self.cookies.push(Cookie::parse(cookie).unwrap());
    }

    pub fn create_cookie_header(&self) -> String {
        let mut cookie_header = String::new();
        for cookie in &self.cookies {
            cookie_header.push_str(&cookie.to_string());
            cookie_header.push_str("; ");
        }
        cookie_header
    }
}
