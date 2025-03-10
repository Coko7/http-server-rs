use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl HttpHeader {
    pub fn new(name: &str, value: &str) -> Self {
        HttpHeader {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}
