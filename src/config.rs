use reqwest::Client as HttpClient;
use reqwest::header::{HeaderMap, HeaderName};
use url::Url;

use crate::error::ResponsesError;
use crate::ids::{OrganizationId, ProjectId, Secret};

pub const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub(crate) const RESPONSES_PATH: &[&str] = &["responses"];
pub(crate) const INPUT_TOKENS_PATH: &[&str] = &["input_tokens"];
pub(crate) const CANCEL_SEGMENT: &str = "cancel";
pub(crate) const RETRY_AFTER_MS: HeaderName = HeaderName::from_static("retry-after-ms");
pub(crate) const API_KEY_HEADER: HeaderName = HeaderName::from_static("api-key");
pub(crate) const ORGANIZATION_HEADER: HeaderName = HeaderName::from_static("openai-organization");
pub(crate) const PROJECT_HEADER: HeaderName = HeaderName::from_static("openai-project");
pub(crate) const DONE: &str = "[DONE]";

pub enum Auth {
    Bearer(Secret),
    ApiKey(Secret),
}

pub struct Config {
    pub base_url: Url,
    pub auth: Auth,
    pub organization: Option<OrganizationId>,
    pub project: Option<ProjectId>,
    pub headers: HeaderMap,
    pub http: HttpClient,
}

impl Config {
    pub fn new(auth: Auth) -> Result<Config, ResponsesError> {
        Ok(Config {
            base_url: Url::parse(OPENAI_BASE_URL)?,
            auth,
            organization: None,
            project: None,
            headers: HeaderMap::new(),
            http: HttpClient::builder().build()?,
        })
    }
}
