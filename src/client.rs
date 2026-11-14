use futures_util::Stream;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client as HttpClient, RequestBuilder};
use serde::de::DeserializeOwned;
use url::Url;

use crate::config::{
    API_KEY_HEADER, Auth, CANCEL_SEGMENT, Config, INPUT_TOKENS_PATH, ORGANIZATION_HEADER,
    PROJECT_HEADER, RESPONSES_PATH,
};
use crate::error::ResponsesError;
use crate::retry_after::RetryAfter;
use crate::ids::{ResponseId, Secret};
use crate::request::{CountInputTokens, CreateResponse};
use crate::response::{ApiErrorBody, Deleted, InputTokens, Response};
use crate::stream::{StreamEvent, events};

pub struct Client {
    http: HttpClient,
    responses_url: Url,
    input_tokens_url: Url,
    headers: HeaderMap,
}

impl Client {
    pub fn new(config: Config) -> Result<Client, ResponsesError> {
        let mut headers = config.headers;
        let (name, secret) = match config.auth {
            Auth::Bearer(Secret(token)) => (AUTHORIZATION, format!("Bearer {token}")),
            Auth::ApiKey(Secret(key)) => (API_KEY_HEADER, key),
        };
        let mut secret = HeaderValue::from_str(&secret)?;
        secret.set_sensitive(true);
        headers.insert(name, secret);
        let scoped = config
            .organization
            .into_iter()
            .map(|id| (ORGANIZATION_HEADER, id.0))
            .chain(config.project.into_iter().map(|id| (PROJECT_HEADER, id.0)));
        for (name, value) in scoped {
            headers.insert(name, HeaderValue::from_str(&value)?);
        }
        let responses_url = endpoint(&config.base_url, RESPONSES_PATH)?;
        Ok(Client {
            http: config.http,
            input_tokens_url: endpoint(&responses_url, INPUT_TOKENS_PATH)?,
            responses_url,
            headers,
        })
    }

    pub async fn create(&self, request: &CreateResponse) -> Result<Response, ResponsesError> {
        let body = CreateResponse {
            stream: None,
            ..request.clone()
        };
        let builder = self.http.post(self.responses_url.clone()).json(&body);
        self.json(builder).await
    }

    pub async fn stream(
        &self,
        request: &CreateResponse,
    ) -> Result<impl Stream<Item = Result<StreamEvent, ResponsesError>> + use<>, ResponsesError>
    {
        let body = CreateResponse {
            stream: Some(true),
            ..request.clone()
        };
        let builder = self.http.post(self.responses_url.clone()).json(&body);
        let response = self.execute(builder).await?;
        Ok(events(response.bytes_stream()))
    }

    pub async fn retrieve(&self, id: &ResponseId) -> Result<Response, ResponsesError> {
        let url = endpoint(&self.responses_url, &[id.0.as_str()])?;
        self.json(self.http.get(url)).await
    }

    pub async fn cancel(&self, id: &ResponseId) -> Result<Response, ResponsesError> {
        let url = endpoint(&self.responses_url, &[id.0.as_str(), CANCEL_SEGMENT])?;
        self.json(self.http.post(url)).await
    }

    pub async fn delete(&self, id: &ResponseId) -> Result<Deleted, ResponsesError> {
        let url = endpoint(&self.responses_url, &[id.0.as_str()])?;
        self.json(self.http.delete(url)).await
    }

    pub async fn input_tokens(
        &self,
        request: &CountInputTokens,
    ) -> Result<InputTokens, ResponsesError> {
        let builder = self.http.post(self.input_tokens_url.clone()).json(request);
        self.json(builder).await
    }

    async fn json<T: DeserializeOwned>(
        &self,
        builder: RequestBuilder,
    ) -> Result<T, ResponsesError> {
        let response = self.execute(builder).await?;
        Ok(serde_json::from_slice(&response.bytes().await?)?)
    }

    async fn execute(&self, builder: RequestBuilder) -> Result<reqwest::Response, ResponsesError> {
        let response = builder.headers(self.headers.clone()).send().await?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let retry_after = RetryAfter::read(response.headers());
        let body = response.text().await?;
        match serde_json::from_str::<ApiErrorBody>(&body) {
            Ok(parsed) => Err(ResponsesError::Api {
                status,
                error: parsed.error,
                retry_after,
            }),
            Err(parse) => Err(ResponsesError::Status {
                status,
                body,
                parse,
                retry_after,
            }),
        }
    }
}

fn endpoint(base: &Url, path: &[&str]) -> Result<Url, ResponsesError> {
    let mut url = base.clone();
    match url.path_segments_mut() {
        Ok(mut segments) => {
            segments.pop_if_empty().extend(path);
        }
        Err(()) => return Err(ResponsesError::CannotBeABase(base.clone())),
    }
    Ok(url)
}
