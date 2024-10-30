use reqwest::{
    header::{CONTENT_TYPE, USER_AGENT},
    RequestBuilder, StatusCode,
};
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

        // For some reason, when unauthorized, the API will redirect to the
        // login page, despite the agent not being a browser. This is annoying,
        // but consistent enough that we can anticipate it and turn it into an
        // error that makes sense.
        if response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|content_type| {
                content_type
                    .to_str()
                    .map(|content_type_str| content_type_str.contains("html"))
                    .ok()
            })
            .unwrap_or_default()
        {
            return Err(Error::Client(StatusCode::UNAUTHORIZED));
        }

        let status = response.status();

        if status.is_client_error() {
            return Err(Error::Client(status));
        }

        response.json::<UntaggedResult<T>>().await?.into()
    }

    /// If there are no members, returns [`Error::Server`] with
    /// [`ServerError::reason`] being "No subscriptions".
    pub async fn members(&self, status: MemberStatus, page: u16) -> Result<Page<Membership>> {
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

    /// If there are no members, returns [`Error::Server`] with
    /// [`ServerError::reason`] being "No supporters".
    pub async fn supporters(&self, page: u16) -> Result<Page<Support>> {
        self.get("/v1/supporters", |request| request.query(&[("page", page)]))
            .await
    }

    pub async fn support(&self, id: u32) -> Result<Support> {
        self.get(&format!("/v1/supporters/{id}"), |request| request)
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
pub struct Page<T> {
    pub current_page: u16,
    pub data: Vec<T>,
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
    #[serde(rename = "subscription_cancelled_on")]
    pub cancelled_on: Option<String>,
    #[serde(rename = "subscription_created_on")]
    pub created_on: String,
    #[serde(rename = "subscription_updated_on")]
    pub updated_on: String,
    #[serde(rename = "subscription_current_period_start")]
    pub current_period_start: String,
    #[serde(rename = "subscription_current_period_end")]
    pub current_period_end: String,
    #[serde(rename = "subscription_coffee_price")]
    pub coffee_price: String,
    #[serde(rename = "subscription_coffee_num")]
    pub coffees: u16,
    #[serde(rename = "subscription_is_cancelled", default)]
    pub is_cancelled: bool,
    #[serde(rename = "subscription_is_cancelled_at_period_end", default)]
    pub is_cancelled_at_period_end: bool,
    #[serde(rename = "subscription_currency")]
    pub currency: String,
    #[serde(rename = "subscription_message")]
    pub message: Option<String>,
    pub message_visibility: u8,
    #[serde(rename = "subscription_duration_type")]
    pub duration_type: String,
    pub referer: Option<String>,
    pub country: Option<String>,
    pub transaction_id: String,
    pub payer_email: String,
    pub payer_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Support {
    #[serde(rename = "support_id")]
    pub id: u32,
    #[serde(rename = "support_note")]
    pub note: Option<String>,
    #[serde(rename = "support_coffees")]
    pub coffees: u16,
    pub transaction_id: String,
    #[serde(rename = "support_visibility")]
    pub visibility: u8,
    #[serde(rename = "support_created_on")]
    pub created_on: String,
    #[serde(rename = "support_updated_on")]
    pub updated_on: String,
    pub transfer_id: Option<String>,
    pub supporter_name: Option<String>,
    #[serde(rename = "support_coffee_price")]
    pub coffee_price: String,
    #[serde(rename = "support_email")]
    pub email: String,
    #[serde(default)]
    pub is_refunded: bool,
    #[serde(rename = "support_currency")]
    pub currency: String,
    #[serde(rename = "support_note_pinned")]
    pub note_pinned: u8,
    pub referer: Option<String>,
    pub country: Option<String>,
    pub payer_email: String,
    pub payment_platform: String,
    pub payer_name: String,
}
