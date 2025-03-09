use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

impl FromStr for SameSitePolicy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "Strict" => SameSitePolicy::Strict,
            "Lax" => SameSitePolicy::Lax,
            "None" => SameSitePolicy::None,
            value => return Err(anyhow!("unknown SameSite policy: {}", value)),
        })
    }
}

impl ToString for SameSitePolicy {
    fn to_string(&self) -> String {
        match self {
            SameSitePolicy::Strict => "Strict",
            SameSitePolicy::Lax => "Lax",
            SameSitePolicy::None => "None",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpCookie {
    name: String,
    value: String,

    domain: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
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
    pub fn new(name: &str, value: &str) -> HttpCookie {
        HttpCookie {
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
        }
    }

    fn get_attribute(attributes: &Vec<String>, attribute: &str) -> Option<String> {
        attributes
            .iter()
            .find(|attr| attr.starts_with(attribute))
            .map(|s| s.to_owned())
    }

    fn get_str_attribute(attributes: &Vec<String>, attribute: &str) -> Option<String> {
        match Self::get_attribute(attributes, attribute) {
            Some(attribute) => Some(attribute.split_once("=")?.1.to_string()),
            None => None,
        }
    }

    fn get_date_attribute(
        attributes: &Vec<String>,
        attribute: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        match Self::get_str_attribute(attributes, attribute) {
            Some(str_val) => match DateTime::parse_from_rfc2822(&str_val) {
                Ok(date) => Ok(Some(date.with_timezone(&Utc))),
                Err(error) => Err(anyhow!(
                    "failed to parse string '{}' to DateTime<Utc>: {}",
                    str_val,
                    error
                )),
            },
            None => Ok(None),
        }
    }

    fn get_i32_attribute(attributes: &Vec<String>, attribute: &str) -> Result<Option<i32>> {
        match Self::get_str_attribute(attributes, attribute) {
            Some(str_val) => match str_val.parse::<i32>() {
                Ok(num) => Ok(Some(num)),
                Err(error) => Err(anyhow!(
                    "failed to parse string '{}' to i32: {}",
                    str_val,
                    error
                )),
            },
            None => Ok(None),
        }
    }

    fn get_bool_attribute(attributes: &Vec<String>, attribute: &str) -> bool {
        Self::get_attribute(attributes, attribute).is_some()
    }

    pub fn from_cookie_line(line: &str) -> Result<HttpCookie> {
        let attributes: Vec<_> = line
            .split(";")
            .map(|attr| attr.trim())
            .map(str::to_string)
            .collect();

        let (name, value) = attributes
            .first()
            .context("")?
            .split_once("=")
            .context("cookie must start with `name=value`")?;

        let domain = Self::get_str_attribute(&attributes, "Domain");
        let expires = Self::get_date_attribute(&attributes, "Expires")?;
        let http_only = Self::get_bool_attribute(&attributes, "HttpOnly");
        let max_age = Self::get_i32_attribute(&attributes, "Max-Age")?;
        let partitioned = Self::get_bool_attribute(&attributes, "Partitioned");
        let path = Self::get_str_attribute(&attributes, "Path");
        let same_site = match Self::get_str_attribute(&attributes, "SameSite") {
            Some(val) => match SameSitePolicy::from_str(&val) {
                Ok(policy) => Ok(Some(policy)),
                Err(error) => Err(error),
            },
            None => Ok(None),
        }?;
        let secure = Self::get_bool_attribute(&attributes, "Secure");

        Ok(HttpCookie {
            name: name.to_string(),
            value: value.to_string(),
            domain,
            expires,
            http_only,
            max_age,
            partitioned,
            path,
            same_site,
            secure,
        })
    }

    pub fn set_domain(mut self, domain: Option<String>) -> Self {
        self.domain = domain;
        self
    }

    pub fn set_expires(mut self, expires: Option<DateTime<Utc>>) -> Self {
        self.expires = expires;
        self
    }

    pub fn set_http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    pub fn set_max_age(mut self, max_age: Option<i32>) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn set_partitioned(mut self, partitioned: bool) -> Self {
        self.partitioned = partitioned;
        self
    }

    pub fn set_path(mut self, path: Option<String>) -> Self {
        self.path = path;
        self
    }

    pub fn set_same_site(mut self, same_site: Option<SameSitePolicy>) -> Self {
        self.same_site = same_site;
        self
    }

    pub fn set_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    fn validate(&self) -> Result<()> {
        if !is_name_valid(&self.name) {
            return Err(anyhow!("invalid characters in cookie name. See MDN: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie"));
        }

        if !is_value_valid(&self.value) {
            return Err(anyhow!("invalid characters in cookie value. See MDN: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie"));
        }

        if let Some(same_site) = self.same_site {
            if same_site == SameSitePolicy::None && !self.secure {
                return Err(anyhow!("cookie with `SameSite=None` must have `Secure`"));
            }
        }

        Ok(())
    }

    pub fn to_str(&self) -> Result<String> {
        self.validate()?;

        let mut attributes = Vec::new();
        attributes.push(format!("{}={}", self.name, self.value));

        if let Some(domain) = &self.domain {
            attributes.push(format!("Domain={}", domain));
        }

        if let Some(expires) = &self.expires {
            let expires = expires.to_rfc2822();
            attributes.push(format!("Expires={}", expires));
        }

        if self.http_only {
            attributes.push("HttpOnly".to_string());
        }

        if let Some(max_age) = &self.max_age {
            attributes.push(format!("Max-Age={}", max_age));
        }

        if self.partitioned {
            attributes.push("Partitioned".to_string());
        }

        if let Some(path) = &self.path {
            attributes.push(format!("Path={}", path));
        }

        if let Some(same_site) = &self.same_site {
            attributes.push(format!("SameSite={}", same_site.to_string()));
        }

        if self.secure {
            attributes.push("Secure".to_string());
        }

        Ok(attributes.join("; "))
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
        let actual = HttpCookie::new("foo", "bar").to_str().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_name_illegal() {
        assert!(HttpCookie::new("f<oo", "bar").to_str().is_err())
    }

    #[test]
    fn test_cookie_value_illegal() {
        assert!(HttpCookie::new("foo", "b,ar").to_str().is_err())
    }

    #[test]
    fn test_cookie_attr_domain() {
        let expected = "foo=bar; Domain=example.com";
        let actual = HttpCookie::new("foo", "bar")
            .set_domain(Some("example.com".to_string()))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_expires() {
        let expires_str = "Tue, 29 Oct 2024 16:56:32 +0000";
        let expires = DateTime::parse_from_rfc2822(expires_str)
            .unwrap()
            .with_timezone(&Utc);

        let expected = format!("foo=bar; Expires={}", expires_str);
        let actual = HttpCookie::new("foo", "bar")
            .set_expires(Some(expires))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_http_only_true() {
        let expected = "foo=bar; HttpOnly";
        let actual = HttpCookie::new("foo", "bar")
            .set_http_only(true)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_http_only_false() {
        let expected = "foo=bar";
        let actual = HttpCookie::new("foo", "bar")
            .set_http_only(false)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_max_age_negative() {
        let expected = "foo=bar; Max-Age=-1";
        let actual = HttpCookie::new("foo", "bar")
            .set_max_age(Some(-1))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_max_age_zero() {
        let expected = "foo=bar; Max-Age=0";
        let actual = HttpCookie::new("foo", "bar")
            .set_max_age(Some(0))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_max_age_positive() {
        let expected = "foo=bar; Max-Age=31536000";
        let actual = HttpCookie::new("foo", "bar")
            .set_max_age(Some(31_536_000))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_partitioned_true() {
        let expected = "foo=bar; Partitioned";
        let actual = HttpCookie::new("foo", "bar")
            .set_partitioned(true)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_partitioned_false() {
        let expected = "foo=bar";
        let actual = HttpCookie::new("foo", "bar")
            .set_partitioned(false)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_path() {
        let expected = "foo=bar; Path=/foo/bar/baz";
        let actual = HttpCookie::new("foo", "bar")
            .set_path(Some("/foo/bar/baz".to_string()))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_same_site_strict() {
        let expected = "foo=bar; SameSite=Strict";
        let actual = HttpCookie::new("foo", "bar")
            .set_same_site(Some(SameSitePolicy::Strict))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_same_site_lax() {
        let expected = "foo=bar; SameSite=Lax";
        let actual = HttpCookie::new("foo", "bar")
            .set_same_site(Some(SameSitePolicy::Lax))
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_same_site_none_secure() {
        let expected = "foo=bar; SameSite=None; Secure";
        let actual = HttpCookie::new("foo", "bar")
            .set_same_site(Some(SameSitePolicy::None))
            .set_secure(true)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_same_site_none_nosecure_err() {
        let actual = HttpCookie::new("foo", "bar")
            .set_same_site(Some(SameSitePolicy::None))
            .to_str();

        assert!(actual.is_err());
    }

    #[test]
    fn test_cookie_attr_secure_true() {
        let expected = "foo=bar; Secure";
        let actual = HttpCookie::new("foo", "bar")
            .set_secure(true)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_secure_false() {
        let expected = "foo=bar";
        let actual = HttpCookie::new("foo", "bar")
            .set_secure(false)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_attr_multiple() {
        let expected = "foo=bar; Domain=example.com; HttpOnly; Secure";
        let actual = HttpCookie::new("foo", "bar")
            .set_domain(Some("example.com".to_string()))
            .set_secure(true)
            .set_http_only(true)
            .to_str()
            .unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_from_line_missing_name_val_err() {
        let cookie_line = "HttpOnly; Max-Age=3600";
        assert!(HttpCookie::from_cookie_line(cookie_line).is_err());
    }

    #[test]
    fn test_cookie_from_line_ok() {
        let expires = DateTime::parse_from_rfc2822("Tue, 29 Oct 2024 16:56:32 +0000")
            .unwrap()
            .with_timezone(&Utc);

        let expected = HttpCookie::new("foo", "bar")
            .set_domain(Some("example.com".to_string()))
            .set_expires(Some(expires))
            .set_http_only(true)
            .set_max_age(Some(3600))
            .set_partitioned(true)
            .set_path(Some("/some/path".to_string()))
            .set_secure(true)
            .set_same_site(Some(SameSitePolicy::Strict));

        let cookie_line = "foo=bar; Domain=example.com; \
            Expires=Tue, 29 Oct 2024 16:56:32 +0000; HttpOnly; Max-Age=3600; \
            Partitioned; Path=/some/path; Secure; SameSite=Strict";
        let actual = HttpCookie::from_cookie_line(cookie_line).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cookie_from_line_unknown_attr_ok() {
        let expires = DateTime::parse_from_rfc2822("Tue, 29 Oct 2024 16:56:32 +0000")
            .unwrap()
            .with_timezone(&Utc);

        let expected = HttpCookie::new("foo", "bar")
            .set_domain(Some("example.com".to_string()))
            .set_expires(Some(expires))
            .set_http_only(true)
            .set_max_age(Some(3600))
            .set_partitioned(true)
            .set_path(Some("/some/path".to_string()))
            .set_secure(true)
            .set_same_site(Some(SameSitePolicy::Strict));

        let cookie_line = "foo=bar; Domain=example.com; \
            Expires=Tue, 29 Oct 2024 16:56:32 +0000; HttpOnly; Max-Age=3600; \
            SomeUnknownAttribute=BAZ; \
            Partitioned; Path=/some/path; Secure; SameSite=Strict";

        let actual = HttpCookie::from_cookie_line(cookie_line).unwrap();

        assert_eq!(expected, actual);
    }
}
