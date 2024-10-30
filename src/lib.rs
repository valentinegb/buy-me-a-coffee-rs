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
#[error("{}{reason}", if let Some(error_code) = .error_code { format!("{error_code} ") } else { "".to_string() })]
pub struct ServerError {
    pub error_code: Option<u16>,
    #[serde(alias = "error")]
    pub reason: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UntaggedResult<T> {
    Err(ServerError),
    Ok(T),
}

impl<T> Into<Result<T>> for UntaggedResult<T> {
    fn into(self) -> Result<T> {
        match self {
            UntaggedResult::Err(err) => Err(Error::Server(err)),
            UntaggedResult::Ok(t) => Ok(t),
        }
    }
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
            return Err(Error::Client(status));
        }

        response.json::<UntaggedResult<T>>().await?.into()
    }

    pub async fn members(&self, status: MemberStatus, page: u16) -> Result<Members> {
        self.get("/v1/subscriptions", |request| {
            request
                .query(&[("status", status)])
                .query(&[("page", page)])
        })
        .await
    }

    pub async fn membership(&self, id: u32) -> Result<Membership> {
        self.get(&format!("/v1/subscriptions/{id}"), |request| request)
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
pub struct Members {
    pub current_page: u16,
    pub data: Vec<Membership>,
    pub from: u16,
    pub last_page: u16,
    pub per_page: u16,
    pub to: u16,
    pub total: u16,
}

#[derive(Debug, Deserialize)]
pub struct Membership {
    #[serde(rename = "subscription_id")]
    pub id: u32,
}
