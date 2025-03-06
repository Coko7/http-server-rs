use std::hash::{Hash, Hasher};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

pub struct HttpCookie {
    name: String,
    value: String,

    domain: Option<String>,
    expires: Option<DateTime<Utc>>,
    http_only: bool,
    max_age: Option<i32>,
    partitioned: bool,
    path: Option<String>,
    same_site: Option<SameSitePolicy>,
    secure: bool,
}

impl Hash for HttpCookie {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

const BANNED_NAME_CHARS: &str = "()<>@,;:\\\"/[]?={}";
const BANNED_VALUE_CHARS: &str = "\"',;\\";

impl HttpCookie {
    pub fn new(name: &str, value: &str) -> Result<HttpCookie> {
        if !is_name_valid(name) {
            return Err(anyhow!("invalid characters in cookie name. See MDN: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie"));
        }

        if !is_value_valid(value) {
            return Err(anyhow!("invalid characters in cookie value. See MDN: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie"));
        }

        Ok(HttpCookie {
            name: name.to_string(),
            value: value.to_string(),
            domain: None,
            expires: None,
            http_only: false,
            max_age: None,
            partitioned: false,
            path: None,
            same_site: None,
            secure: false,
        })
    }

    pub fn to_str(&self) -> String {
        let mut result = String::new();
        let name_val = format!("{}={}", self.name, self.value);
        result.push_str(&name_val);

        result
    }
}

fn is_name_valid(cookie_name: &str) -> bool {
    let has_illegal_chars = cookie_name.chars().any(|ch| {
        ch as u8 <= 31 || ch as u8 >= 127 || BANNED_NAME_CHARS.contains(ch) || ch.is_whitespace()
    });

    cookie_name.is_ascii() && !has_illegal_chars
}

fn is_value_valid(cookie_value: &str) -> bool {
    // Remove first and last char if double quotes (allow to be wrapped in double quotes)
    let cookie_value = if cookie_value.starts_with("\"") && cookie_value.ends_with("\"") {
        &cookie_value[1..cookie_value.len() - 1]
    } else {
        cookie_value
    };

    let has_illegal_chars = cookie_value.chars().any(|ch| {
        ch as u8 <= 31 || ch as u8 >= 127 || BANNED_VALUE_CHARS.contains(ch) || ch.is_whitespace()
    });

    cookie_value.is_ascii() && !has_illegal_chars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie() {
        let expected = "foo=bar";
        let actual = HttpCookie::new("foo", "bar").unwrap();
        assert_eq!(expected, actual.to_str());
    }

    #[test]
    fn test_cookie_name_illegal() {
        assert!(HttpCookie::new("f<oo", "bar").is_err())
    }

    #[test]
    fn test_cookie_value_illegal() {
        assert!(HttpCookie::new("foo", "b,ar").is_err())
    }

    #[test]
    fn test_cookie_domain() {
        let expected = "foo=bar; Domain=example.com";
        let mut actual = HttpCookie::new("foo", "bar").unwrap();
        actual.domain = Some("example.com".to_string());

        assert_eq!(expected, actual.to_str());
    }

    // #[test]
    // fn test_cookie_expire() {
    //     let expected = "foo=bar; Expires=Tue, 29 Oct 2024 16:56:32 GMT";
    //     let mut actual = HttpCookie::new("foo", "bar").unwrap();
    //     actual.expires = Some("example.com".to_string());
    //
    //     assert_eq!(expected, actual.to_str());
    // }
}
