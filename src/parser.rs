use crate::parser;
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

    pub fn get_form_token(&self) -> Result<String, Box<dyn Error>> {
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
}

pub fn parse_cookies(cookie_str: &str) -> HashMap<String, String> {
    cookie_str
        .split(';')
        .into_iter()
        .filter_map(|c| {
            if let Some((name, value)) = c.trim().split_once('=') {
                Some((name.to_owned(), value.to_owned()))
            } else {
                None
            }
        })
        .collect()
}

pub async fn get_session_id<'a>(headers: HeaderMap) -> Result<String, Box<dyn Error>> {
    match headers.get("Set-Cookie") {
        Some(cookie) => match cookie.to_str() {
            Ok(cookie_str) => parser::parse_cookies(cookie_str)
                .remove("sessionid")
                .ok_or("sessionid not found in cookie_str".into()),
            Err(e) => Err(e.into()),
        },
        None => Err("Cookie not found".into()),
    }
}
