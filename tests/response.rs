use openai_responses_api::{
    AnnotationKind, ApiErrorBody, ApiErrorKind, Content, Deleted, ErrorCode, IncompleteReason,
    InputTokens, Item, ItemStatus, MessagePhase, Open, Param, Part, Response, ResponseId,
    ResponseStatus, ResponsesError, SearchAction, SearchSource, ServiceTier,
};
use serde_json::json;

fn parse(raw: &str) -> Result<Response, ResponsesError> {
    Ok(serde_json::from_str(raw)?)
}

#[test]
fn full_response_parses_and_keeps_unknowns() -> Result<(), ResponsesError> {
    let response = parse(include_str!("fixtures/response.json"))?;
    assert_eq!(response.id, ResponseId("resp_1".to_owned()));
    assert_eq!(response.status, ResponseStatus::Completed);
    assert_eq!(response.output_text(), "Hello world.");
    assert!(
        matches!(&response.output[0], Open::Known(Item::Reasoning(r)) if r.status == Some(ItemStatus::Completed) && r.summary.len() == 1)
    );
    let Open::Known(Item::WebSearchCall(search)) = &response.output[1] else {
        panic!("second item is not a web search call");
    };
    let Some(Open::Known(SearchAction::Search { query, sources })) = &search.action else {
        panic!("web search action is not a search");
    };
    assert_eq!(query.as_deref(), Some("rust serde"));
    assert!(
        matches!(sources.as_deref(), Some([Open::Known(SearchSource::Url { url })]) if url.as_str() == "https://serde.rs/")
    );
    let Open::Known(Item::Message(message)) = &response.output[2] else {
        panic!("third item is not a message");
    };
    assert_eq!(message.phase, Some(MessagePhase::FinalAnswer));
    let Content::Parts(parts) = &message.content else {
        panic!("message content is not parts");
    };
    let Open::Known(Part::OutputText(first)) = &parts[0] else {
        panic!("first part is not output text");
    };
    assert!(matches!(
        &first.annotations[0],
        Open::Known(AnnotationKind::UrlCitation { end_index: 5, .. })
    ));
    assert!(matches!(&first.annotations[1], Open::Unknown(raw) if raw["type"] == "file_path"));
    let Open::Known(Part::OutputText(second)) = &parts[1] else {
        panic!("second part is not output text");
    };
    let logprobs = second.logprobs.as_deref().unwrap_or_default();
    assert_eq!(logprobs[0].bytes.as_deref(), Some("world".as_bytes()));
    assert!(
        matches!(&response.output[3], Open::Known(Item::FunctionCall(c)) if c.arguments.0 == "{\"path\":\"notes.txt\"}")
    );
    assert!(matches!(&response.output[4], Open::Unknown(raw) if raw["type"] == "mcp_call"));

    let usage = response.usage.as_ref().map(|u| {
        (
            u.input_tokens,
            u.input_tokens_details.as_ref().map(|d| d.cached_tokens),
            u.output_tokens_details.as_ref().map(|d| d.reasoning_tokens),
            u.total_tokens,
        )
    });
    assert_eq!(usage, Some((120, Some(100), Some(12), 150)));
    assert_eq!(response.service_tier, Some(ServiceTier::Default));
    assert_eq!(response.extra.get("store"), Some(&json!(false)));
    assert_eq!(response.extra.get("object"), Some(&json!("response")));
    Ok(())
}

#[test]
fn response_round_trips() -> Result<(), ResponsesError> {
    let response = parse(include_str!("fixtures/response.json"))?;
    let again: Response = serde_json::from_value(serde_json::to_value(&response)?)?;
    assert_eq!(again, response);
    Ok(())
}

#[test]
fn incomplete_failed_and_unknown_statuses() -> Result<(), ResponsesError> {
    let incomplete = parse(include_str!("fixtures/response_incomplete.json"))?;
    assert_eq!(incomplete.status, ResponseStatus::Incomplete);
    assert_eq!(
        incomplete.incomplete_details.map(|d| d.reason),
        Some(IncompleteReason::MaxOutputTokens)
    );
    let failed = parse(include_str!("fixtures/response_failed.json"))?;
    assert_eq!(failed.status, ResponseStatus::Failed);
    assert_eq!(failed.error.map(|e| e.code), Some(ErrorCode::ServerError));
    let other = parse(include_str!("fixtures/response_other_status.json"))?;
    assert_eq!(other.status, ResponseStatus::Other("paused".to_owned()));
    assert_eq!(other.output_text(), "plain text reply");
    Ok(())
}

#[test]
fn malformed_known_item_is_an_error() {
    let parsed = parse(include_str!("fixtures/response_malformed_call.json"));
    assert!(
        matches!(parsed, Err(ResponsesError::Json(..))),
        "{parsed:?}"
    );
}

#[test]
fn item_without_type_or_role_is_an_error() {
    let message = match parse(include_str!("fixtures/response_untyped_item.json")) {
        Ok(response) => panic!("parsed an untyped item: {response:?}"),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("type tag"), "{message}");
}

#[test]
fn api_error_bodies_parse() -> Result<(), ResponsesError> {
    let openai: ApiErrorBody = serde_json::from_str(include_str!("fixtures/error.json"))?;
    assert_eq!(openai.error.kind, Some(ApiErrorKind::InvalidRequestError));
    assert_eq!(openai.error.code, Some(ErrorCode::InvalidApiKey));
    assert_eq!(openai.error.param, None);
    let numeric: ApiErrorBody = serde_json::from_str(include_str!("fixtures/error_numeric.json"))?;
    assert_eq!(
        numeric.error.kind,
        Some(ApiErrorKind::Other("BadRequestError".to_owned()))
    );
    assert_eq!(numeric.error.code, Some(ErrorCode::Numeric(400)));
    assert_eq!(
        numeric.error.param,
        Some(Param("max_output_tokens".to_owned()))
    );
    Ok(())
}

#[test]
fn input_tokens_and_deleted_parse() -> Result<(), ResponsesError> {
    let count: InputTokens = serde_json::from_str(include_str!("fixtures/input_tokens.json"))?;
    assert_eq!(count.input_tokens, 321);
    let deleted: Deleted = serde_json::from_str(include_str!("fixtures/deleted.json"))?;
    assert!(deleted.deleted);
    assert_eq!(deleted.id, ResponseId("resp_1".to_owned()));
    Ok(())
}
