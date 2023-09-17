use crate::parser::{self, DocumentExtractor};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthForm {
    pub token: String,
    pub password_hash: String,
    pub username: String,
    pub password: String,
    pub remember: String,
}

pub struct Kf2Logger {
    base_url: Url,
    session: Client,
    session_id: String,
}

impl Kf2Logger {
    pub async fn new_session(
        ip_addr: String,
        username: String,
        password: String,
    ) -> Result<Self, Box<dyn Error>> {
        let base_url = Url::parse(&ip_addr).unwrap();
        drop(ip_addr);
        let web_admin_url = base_url.join("ServerAdmin/")?;
        let client = reqwest::ClientBuilder::new().cookie_store(true).build()?;
        let get_response = client.get(web_admin_url.as_str()).send().await?;
        let headers = get_response.headers().clone();
        let document = DocumentExtractor::new(&get_response.text().await?);
        let token = document.get_form_token()?;
        let session_id = parser::get_session_id(headers).await?;

        let form = AuthForm {
            token,
            password_hash: "".to_string(),
            username,
            password,
            remember: "-1".to_string(),
        };

        client.post(web_admin_url).form(&form).send().await?;
        let session = client;

        Ok(Self {
            base_url,
            session,
            session_id,
        })
    }

    pub async fn log_all(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    pub async fn log_players(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
