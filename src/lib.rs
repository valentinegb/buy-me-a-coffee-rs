use reqwest::{header::USER_AGENT, RequestBuilder, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

const PREFIX: &str = "https://developers.buymeacoffee.com/api";
const USER_AGENT_VALUE: &str = "buy-me-a-coffee-rs/0.1.0";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Client(StatusCode),
    #[error(transparent)]
    Server(#[from] ServerError),
}

#[derive(Debug, Error, Deserialize)]
#[error("{error_code} {reason}")]
pub struct ServerError {
    pub error_code: u16,
    pub reason: String,
}

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    token: String,
}

impl Client {
    pub fn new(token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: token.to_string(),
        }
    }

    async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        modify_request: impl FnOnce(RequestBuilder) -> RequestBuilder,
    ) -> Result<T> {
        let mut request = self
            .client
            .get(format!("{PREFIX}{endpoint}"))
            .bearer_auth(&self.token)
            .header(USER_AGENT, USER_AGENT_VALUE);

        request = modify_request(request);

        let response = request.send().await?;
        let status = response.status();

        if status.is_client_error() {
            Err(Error::Client(status))
        } else if status.is_server_error() {
            Err(Error::Server(response.json().await?))
        } else {
            Ok(response.json().await?)
        }
    }

    pub async fn members(&self, status: MemberStatus) -> Result<Members> {
        self.get("/v1/subscriptions", |request| {
            request.query(&[("status", status)])
        })
        .await
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MemberStatus {
    Active,
    Inactive,
    All,
}

#[derive(Debug, Deserialize)]
pub struct Members {}
