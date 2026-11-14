use futures::executor::block_on;
use futures::stream::{self, StreamExt};
use openai_responses_api::{
    ErrorCode, Event, Item, Open, ResponseStatus, ResponsesError, StreamEvent, events,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct ReadFile {
    path: String,
}

fn parse(sse: &'static str, chunk: usize) -> Vec<Result<StreamEvent, ResponsesError>> {
    let chunks = sse
        .as_bytes()
        .chunks(chunk)
        .map(Ok::<&[u8], reqwest::Error>);
    block_on(events(stream::iter(chunks)).collect())
}

fn all(sse: &'static str, chunk: usize) -> Result<Vec<StreamEvent>, ResponsesError> {
    parse(sse, chunk).into_iter().collect()
}

fn kind(event: &StreamEvent) -> &str {
    match event {
        Open::Known(Event::Created(..)) => "created",
        Open::Known(Event::Queued(..)) => "queued",
        Open::Known(Event::InProgress(..)) => "in_progress",
        Open::Known(Event::Completed(..)) => "completed",
        Open::Known(Event::Failed(..)) => "failed",
        Open::Known(Event::Incomplete(..)) => "incomplete",
        Open::Known(Event::OutputItemAdded(..)) => "item_added",
        Open::Known(Event::OutputItemDone(..)) => "item_done",
        Open::Known(Event::ContentPartAdded(..)) => "part_added",
        Open::Known(Event::ContentPartDone(..)) => "part_done",
        Open::Known(Event::OutputTextDelta(..)) => "text_delta",
        Open::Known(Event::OutputTextDone(..)) => "text_done",
        Open::Known(Event::AnnotationAdded(..)) => "annotation",
        Open::Known(Event::RefusalDelta(..)) => "refusal_delta",
        Open::Known(Event::RefusalDone(..)) => "refusal_done",
        Open::Known(Event::FunctionCallArgumentsDelta(..)) => "args_delta",
        Open::Known(Event::FunctionCallArgumentsDone(..)) => "args_done",
        Open::Known(Event::CustomToolCallInputDelta(..)) => "custom_delta",
        Open::Known(Event::CustomToolCallInputDone(..)) => "custom_done",
        Open::Known(Event::ReasoningSummaryPartAdded(..)) => "summary_part_added",
        Open::Known(Event::ReasoningSummaryPartDone(..)) => "summary_part_done",
        Open::Known(Event::ReasoningSummaryTextDelta(..)) => "summary_delta",
        Open::Known(Event::ReasoningSummaryTextDone(..)) => "summary_done",
        Open::Known(Event::ReasoningTextDelta(..)) => "reasoning_delta",
        Open::Known(Event::ReasoningTextDone(..)) => "reasoning_done",
        Open::Known(Event::WebSearchCallInProgress(..)) => "search_in_progress",
        Open::Known(Event::WebSearchCallSearching(..)) => "search_searching",
        Open::Known(Event::WebSearchCallCompleted(..)) => "search_completed",
        Open::Known(Event::Error(..)) => "error",
        Open::Unknown(..) => "unknown",
    }
}

#[test]
fn full_stream_parses_at_any_chunking() -> Result<(), ResponsesError> {
    let expected = [
        "created",
        "in_progress",
        "item_added",
        "summary_part_added",
        "summary_delta",
        "summary_done",
        "summary_part_done",
        "item_done",
        "item_added",
        "part_added",
        "text_delta",
        "text_delta",
        "annotation",
        "text_done",
        "part_done",
        "item_done",
        "item_added",
        "args_delta",
        "args_delta",
        "args_done",
        "item_done",
        "item_added",
        "custom_delta",
        "custom_done",
        "search_searching",
        "unknown",
        "completed",
    ];
    for chunk in [1, 7, 64, 100000] {
        let events = all(include_str!("fixtures/stream.sse"), chunk)?;
        let kinds: Vec<&str> = events.iter().map(kind).collect();
        assert_eq!(kinds, expected, "chunk size {chunk}");
    }
    Ok(())
}

#[test]
fn deltas_rebuild_text_and_arguments() -> Result<(), ResponsesError> {
    let events = all(include_str!("fixtures/stream.sse"), 64)?;
    let mut text = String::new();
    let mut arguments = String::new();
    for event in &events {
        match event {
            Open::Known(Event::OutputTextDelta(delta)) => text.push_str(&delta.delta),
            Open::Known(Event::FunctionCallArgumentsDelta(delta)) => {
                arguments.push_str(&delta.delta)
            }
            Open::Known(..) | Open::Unknown(..) => {}
        }
    }
    assert_eq!(text, "Hi there");
    let Some(Open::Known(Event::FunctionCallArgumentsDone(done))) = events.get(19) else {
        panic!("event 19 is not arguments done");
    };
    assert_eq!(done.arguments.0, arguments);
    assert_eq!(done.arguments.parse::<ReadFile>()?.path, "a");
    let Some(Open::Known(Event::Completed(completed))) = events.last() else {
        panic!("last event is not completed");
    };
    assert_eq!(completed.response.status, ResponseStatus::Completed);
    assert_eq!(completed.response.output_text(), "Hi there");
    assert!(matches!(
        &completed.response.output[2],
        Open::Known(Item::FunctionCall(..))
    ));
    assert_eq!(completed.sequence_number, 26);
    Ok(())
}

#[test]
fn error_event_is_typed() -> Result<(), ResponsesError> {
    let events = all(include_str!("fixtures/stream_error.sse"), 7)?;
    let Some(Open::Known(Event::Error(error))) = events.last() else {
        panic!("last event is not an error");
    };
    assert_eq!(error.code, Some(ErrorCode::RateLimitExceeded));
    assert_eq!(error.message, "slow down");
    Ok(())
}

#[test]
fn done_sentinel_is_kept_as_unknown() -> Result<(), ResponsesError> {
    let events = all(include_str!("fixtures/stream_done.sse"), 5)?;
    let kinds: Vec<&str> = events.iter().map(kind).collect();
    assert_eq!(kinds, ["created", "completed", "unknown"]);
    assert!(matches!(events.last(), Some(Open::Unknown(Value::String(s))) if s == "[DONE]"));
    Ok(())
}

#[test]
fn malformed_known_event_is_an_error() {
    let results = parse(include_str!("fixtures/stream_malformed.sse"), 100000);
    assert!(
        matches!(results.as_slice(), [Err(ResponsesError::Json(..))]),
        "{results:?}"
    );
}
