use crate::parse::{DocumentExtractor, HeaderExtractor};
use reqwest::{Client, ClientBuilder};
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
    web_admin_url: Url,
    session: Client,
    session_id: String,
    auth_cred: String,
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
        let client = ClientBuilder::new().cookie_store(true).build()?;
        let get_response = client.get(web_admin_url.as_str()).send().await?;
        let headers = HeaderExtractor::new(get_response.headers().to_owned());
        let document = DocumentExtractor::new(&get_response.text().await?);
        let token = document.parse_form_token()?;
        let session_id = headers.get_cookie("sessionid").unwrap();
        let auth_cred = headers.get_cookie("authcred").unwrap();

        let form = AuthForm {
            token,
            password_hash: "".to_string(),
            username,
            password,
            remember: "-1".to_string(),
        };

        client
            .post(web_admin_url.clone())
            .form(&form)
            .send()
            .await?;
        let session = client;

        Ok(Self {
            base_url,
            web_admin_url,
            session,
            session_id,
            auth_cred,
        })
    }

    async fn get_info(&self) -> Result<(), Box<dyn Error>> {
        let info_url = self.web_admin_url.join("info")?;
        let response = self.session.get(info_url).send().await?;
        let document = DocumentExtractor::new(&response.text().await?);
        let info = document.parse_info()?;
        todo!()
    }

    pub async fn log_all(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    pub async fn log_players(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
