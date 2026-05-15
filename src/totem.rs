use reqwest::{Client, StatusCode};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TotemState {
    FocusOn,
    FocusOff,
    Error,
}

#[derive(Debug)]
pub enum TotemPollError {
    HttpStatus(StatusCode),
    InvalidPayload(String),
    Request(reqwest::Error),
}

impl fmt::Display for TotemPollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpStatus(status) => write!(f, "HTTP status {status}"),
            Self::InvalidPayload(payload) => write!(f, "unexpected status payload: {payload:?}"),
            Self::Request(error) => write!(f, "{error}"),
        }
    }
}

pub async fn fetch_state(
    http_client: &Client,
    address: &str,
) -> Result<TotemState, TotemPollError> {
    match http_client.get(address).send().await {
        Ok(response) if response.status().is_success() => {
            let body = response.text().await.map_err(TotemPollError::Request)?;
            parse_totem_state(&body)
        }
        Ok(response) => Err(TotemPollError::HttpStatus(response.status())),
        Err(e) => Err(TotemPollError::Request(e)),
    }
}

fn parse_totem_state(body: &str) -> Result<TotemState, TotemPollError> {
    match body.trim() {
        "FOCUS_ON" => Ok(TotemState::FocusOn),
        "FOCUS_OFF" => Ok(TotemState::FocusOff),
        other => Err(TotemPollError::InvalidPayload(other.to_string())),
    }
}
