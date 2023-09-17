use crate::parse;
use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use std::{collections::HashMap, error::Error};

pub struct DocumentExtractor {
    document: Html,
}

impl DocumentExtractor {
    pub fn new(document: &str) -> Self {
        Self {
            document: Html::parse_document(document),
        }
    }

    pub fn parse_form_token(&self) -> Result<String, Box<dyn Error>> {
        let selector = Selector::parse(r#"input[name="token"]"#)?;
        let token = self
            .document
            .select(&selector)
            .next()
            .ok_or("Token not found")?;
        let value = token
            .value()
            .attr("value")
            .ok_or("Token value field not found")?;
        Ok(value.to_string())
    }

    pub fn parse_info(&self) -> Result<String, Box<dyn Error>> {
        todo!()
    }
}

pub struct HeaderExtractor {
    headers: HeaderMap,
    cookies: HashMap<String, String>,
}

impl HeaderExtractor {
    pub fn new(headers: HeaderMap) -> Self {
        let cookies = Self::parse_cookies(headers.clone());
        Self { headers, cookies }
    }

    fn parse_cookies(headers: HeaderMap) -> HashMap<String, String> {
        match headers.get("Set-Cookie") {
            Some(cookie) => match cookie.to_str() {
                Ok(cookie_str) => cookie_str
                    .split(';')
                    .into_iter()
                    .filter_map(|c| {
                        if let Some((name, value)) = c.trim().split_once('=') {
                            Some((name.to_owned(), value.to_owned()))
                        } else {
                            None
                        }
                    })
                    .collect(),
                Err(e) => HashMap::new(),
            },
            None => HashMap::new(),
        }
    }

    pub fn get_cookie(&self, cookie: &str) -> Option<String> {
        self.cookies.get(cookie).cloned()
    }
}
