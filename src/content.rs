use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::error::ResponsesError;
use crate::ids::{FileData, FileId, FileName};
use crate::open::{Kinds, Open};

pub type ContentPart = Open<Part>;
pub type Annotation = Open<AnnotationKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
    System,
    Developer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, try_from = "Value")]
pub enum Content {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl TryFrom<Value> for Content {
    type Error = ResponsesError;

    fn try_from(value: Value) -> Result<Content, ResponsesError> {
        match value {
            Value::String(text) => Ok(Content::Text(text)),
            parts => Ok(Content::Parts(serde_json::from_value(parts)?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Part {
    InputText { text: String },
    InputImage(InputImage),
    InputFile(InputFile),
    OutputText(OutputText),
    Refusal { refusal: String },
    SummaryText { text: String },
    ReasoningText { text: String },
}

impl Kinds for Part {
    const KINDS: &'static [&'static str] = &[
        "input_text",
        "input_image",
        "input_file",
        "output_text",
        "refusal",
        "summary_text",
        "reasoning_text",
    ];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputImage {
    #[serde(flatten)]
    pub source: ImageSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSource {
    Url { image_url: Url },
    File { file_id: FileId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageDetail {
    Low,
    High,
    Auto,
    Original,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputFile {
    #[serde(flatten)]
    pub source: FileSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<FileName>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileSource {
    Id { file_id: FileId },
    Url { file_url: Url },
    Data { file_data: FileData },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputText {
    pub text: String,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProb>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnnotationKind {
    UrlCitation {
        url: Url,
        title: String,
        start_index: u32,
        end_index: u32,
    },
    FileCitation {
        file_id: FileId,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<FileName>,
        index: u32,
    },
}

impl Kinds for AnnotationKind {
    const KINDS: &'static [&'static str] = &["url_citation", "file_citation"];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogProb {
    pub token: String,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
    #[serde(default)]
    pub top_logprobs: Vec<TopLogProb>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopLogProb {
    pub token: String,
    pub logprob: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<Vec<u8>>,
}
