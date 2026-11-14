use eventsource_stream::Eventsource;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::DONE;
use crate::content::{Annotation, ContentPart, LogProb};
use crate::error::ResponsesError;
use crate::ids::{Arguments, ItemId, Param};
use crate::item::OutputItem;
use crate::open::{Kinds, Open};
use crate::response::{ErrorCode, Response};

pub type StreamEvent = Open<Event>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "response.created")]
    Created(Lifecycle),
    #[serde(rename = "response.queued")]
    Queued(Lifecycle),
    #[serde(rename = "response.in_progress")]
    InProgress(Lifecycle),
    #[serde(rename = "response.completed")]
    Completed(Lifecycle),
    #[serde(rename = "response.failed")]
    Failed(Lifecycle),
    #[serde(rename = "response.incomplete")]
    Incomplete(Lifecycle),
    #[serde(rename = "response.output_item.added")]
    OutputItemAdded(OutputItemEvent),
    #[serde(rename = "response.output_item.done")]
    OutputItemDone(OutputItemEvent),
    #[serde(rename = "response.content_part.added")]
    ContentPartAdded(ContentPartEvent),
    #[serde(rename = "response.content_part.done")]
    ContentPartDone(ContentPartEvent),
    #[serde(rename = "response.output_text.delta")]
    OutputTextDelta(PartDelta),
    #[serde(rename = "response.output_text.done")]
    OutputTextDone(TextDone),
    #[serde(rename = "response.output_text.annotation.added")]
    AnnotationAdded(AnnotationAdded),
    #[serde(rename = "response.refusal.delta")]
    RefusalDelta(PartDelta),
    #[serde(rename = "response.refusal.done")]
    RefusalDone(RefusalDone),
    #[serde(rename = "response.function_call_arguments.delta")]
    FunctionCallArgumentsDelta(ItemDelta),
    #[serde(rename = "response.function_call_arguments.done")]
    FunctionCallArgumentsDone(ArgumentsDone),
    #[serde(rename = "response.custom_tool_call_input.delta")]
    CustomToolCallInputDelta(ItemDelta),
    #[serde(rename = "response.custom_tool_call_input.done")]
    CustomToolCallInputDone(InputDone),
    #[serde(rename = "response.reasoning_summary_part.added")]
    ReasoningSummaryPartAdded(SummaryPartEvent),
    #[serde(rename = "response.reasoning_summary_part.done")]
    ReasoningSummaryPartDone(SummaryPartEvent),
    #[serde(rename = "response.reasoning_summary_text.delta")]
    ReasoningSummaryTextDelta(SummaryDelta),
    #[serde(rename = "response.reasoning_summary_text.done")]
    ReasoningSummaryTextDone(SummaryDone),
    #[serde(rename = "response.reasoning_text.delta")]
    ReasoningTextDelta(PartDelta),
    #[serde(rename = "response.reasoning_text.done")]
    ReasoningTextDone(TextDone),
    #[serde(rename = "response.web_search_call.in_progress")]
    WebSearchCallInProgress(ItemProgress),
    #[serde(rename = "response.web_search_call.searching")]
    WebSearchCallSearching(ItemProgress),
    #[serde(rename = "response.web_search_call.completed")]
    WebSearchCallCompleted(ItemProgress),
    #[serde(rename = "error")]
    Error(StreamError),
}

impl Kinds for Event {
    const KINDS: &'static [&'static str] = &[
        "response.created",
        "response.queued",
        "response.in_progress",
        "response.completed",
        "response.failed",
        "response.incomplete",
        "response.output_item.added",
        "response.output_item.done",
        "response.content_part.added",
        "response.content_part.done",
        "response.output_text.delta",
        "response.output_text.done",
        "response.output_text.annotation.added",
        "response.refusal.delta",
        "response.refusal.done",
        "response.function_call_arguments.delta",
        "response.function_call_arguments.done",
        "response.custom_tool_call_input.delta",
        "response.custom_tool_call_input.done",
        "response.reasoning_summary_part.added",
        "response.reasoning_summary_part.done",
        "response.reasoning_summary_text.delta",
        "response.reasoning_summary_text.done",
        "response.reasoning_text.delta",
        "response.reasoning_text.done",
        "response.web_search_call.in_progress",
        "response.web_search_call.searching",
        "response.web_search_call.completed",
        "error",
    ];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lifecycle {
    pub response: Response,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputItemEvent {
    pub output_index: u32,
    pub item: OutputItem,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPartEvent {
    pub item_id: ItemId,
    pub output_index: u32,
    pub content_index: u32,
    pub part: ContentPart,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartDelta {
    pub item_id: ItemId,
    pub output_index: u32,
    pub content_index: u32,
    pub delta: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProb>>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextDone {
    pub item_id: ItemId,
    pub output_index: u32,
    pub content_index: u32,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<Vec<LogProb>>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationAdded {
    pub item_id: ItemId,
    pub output_index: u32,
    pub content_index: u32,
    pub annotation_index: u32,
    pub annotation: Annotation,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalDone {
    pub item_id: ItemId,
    pub output_index: u32,
    pub content_index: u32,
    pub refusal: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemDelta {
    pub item_id: ItemId,
    pub output_index: u32,
    pub delta: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArgumentsDone {
    pub item_id: ItemId,
    pub output_index: u32,
    pub arguments: Arguments,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputDone {
    pub item_id: ItemId,
    pub output_index: u32,
    pub input: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryPartEvent {
    pub item_id: ItemId,
    pub output_index: u32,
    pub summary_index: u32,
    pub part: ContentPart,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryDelta {
    pub item_id: ItemId,
    pub output_index: u32,
    pub summary_index: u32,
    pub delta: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryDone {
    pub item_id: ItemId,
    pub output_index: u32,
    pub summary_index: u32,
    pub text: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemProgress {
    pub item_id: ItemId,
    pub output_index: u32,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ErrorCode>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<Param>,
    pub sequence_number: u64,
}

pub fn events<S, B>(bytes: S) -> impl Stream<Item = Result<StreamEvent, ResponsesError>>
where
    S: Stream<Item = Result<B, reqwest::Error>>,
    B: AsRef<[u8]>,
{
    bytes.eventsource().map(|item| match item {
        Ok(event) if event.data == DONE => Ok(Open::Unknown(Value::String(event.data))),
        Ok(event) => serde_json::from_str::<StreamEvent>(&event.data).map_err(ResponsesError::Json),
        Err(e) => Err(ResponsesError::Sse(e)),
    })
}
