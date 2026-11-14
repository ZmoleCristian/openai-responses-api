use openai_responses_api::{
    CallId, Content, CountInputTokens, CreateResponse, FileId, FileSource, FormatKind,
    GrammarSyntax, ImageDetail, ImageSource, Include, Input, Item, LocalShellAction, ModelId,
    NamedTool, Open, Part, ReasoningEffort, ReasoningSummary, ResponsesError, Role, ServiceTier,
    TextFormat, ToolChoice, ToolChoiceMode, ToolKind, ToolName, Truncation, Verbosity,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct ReadFile {
    path: String,
}

fn fixture() -> Result<CreateResponse, ResponsesError> {
    Ok(serde_json::from_str(include_str!("fixtures/request.json"))?)
}

fn items(request: &CreateResponse) -> &[Open<Item>] {
    match &request.input {
        Input::Items(items) => items,
        Input::Text(text) => panic!("expected items, got text {text}"),
    }
}

#[test]
fn codex_style_request_parses() -> Result<(), ResponsesError> {
    let request = fixture()?;
    let input = items(&request);
    assert!(
        matches!(&input[0], Open::Known(Item::Message(m)) if m.role == Role::Developer && m.content == Content::Text("Answer briefly.".to_owned()))
    );
    let Open::Known(Item::Message(user)) = &input[1] else {
        panic!("second item is not a message: {:?}", input[1]);
    };
    let Content::Parts(parts) = &user.content else {
        panic!("user content is not parts");
    };
    assert!(
        matches!(&parts[0], Open::Known(Part::InputText { text }) if text == "What is in this file?")
    );
    assert!(
        matches!(&parts[1], Open::Known(Part::InputImage(i)) if i.detail == Some(ImageDetail::High) && matches!(&i.source, ImageSource::Url { image_url } if image_url.as_str() == "https://example.com/cat.png"))
    );
    assert!(
        matches!(&parts[2], Open::Known(Part::InputFile(f)) if f.source == FileSource::Id { file_id: FileId("file-abc".to_owned()) })
    );
    assert!(
        matches!(&input[2], Open::Known(Item::Reasoning(r)) if r.encrypted_content.as_ref().map(|e| e.0.as_str()) == Some("gAAAAB-opaque"))
    );
    let Open::Known(Item::FunctionCall(call)) = &input[3] else {
        panic!("fourth item is not a function call");
    };
    assert_eq!(call.call_id, CallId("call_1".to_owned()));
    assert_eq!(call.arguments.parse::<ReadFile>()?.path, "notes.txt");
    assert!(
        matches!(&input[4], Open::Known(Item::FunctionCallOutput(o)) if o.output == Content::Text("hello".to_owned()))
    );
    assert!(
        matches!(&input[5], Open::Known(Item::CustomToolCall(c)) if c.name == ToolName("apply_patch".to_owned()))
    );
    assert!(
        matches!(&input[6], Open::Known(Item::CustomToolCallOutput(o)) if matches!(&o.output, Content::Parts(p) if p.len() == 1))
    );
    assert!(
        matches!(&input[7], Open::Known(Item::LocalShellCall(s)) if matches!(&s.action, Open::Known(LocalShellAction::Exec { command, timeout_ms: Some(1000), .. }) if command == &["ls", "-la"]))
    );
    assert!(matches!(&input[8], Open::Known(Item::ItemReference { id }) if id.0 == "msg_0"));
    assert!(matches!(&input[9], Open::Unknown(raw) if raw["type"] == "mcp_approval_response"));

    let tools = request.tools.as_deref().unwrap_or_default();
    assert!(
        matches!(&tools[0], Open::Known(ToolKind::Function(f)) if f.strict == Some(true) && f.parameters.is_some())
    );
    assert!(
        matches!(&tools[1], Open::Known(ToolKind::Custom(c)) if matches!(&c.format, Some(Open::Known(FormatKind::Grammar { syntax: GrammarSyntax::Lark, .. }))))
    );
    assert!(matches!(&tools[2], Open::Known(ToolKind::LocalShell)));
    assert!(
        matches!(&tools[3], Open::Known(ToolKind::WebSearch { config }) if config.get("search_context_size") == Some(&json!("low")))
    );
    assert!(matches!(&tools[4], Open::Unknown(raw) if raw["type"] == "image_generation"));

    assert_eq!(
        request.tool_choice,
        Some(ToolChoice::Mode(ToolChoiceMode::Auto))
    );
    let reasoning = request
        .reasoning
        .as_ref()
        .map(|r| (r.effort.clone(), r.summary.clone()));
    assert_eq!(
        reasoning,
        Some((Some(ReasoningEffort::Xhigh), Some(ReasoningSummary::Auto)))
    );
    let text = request
        .text
        .as_ref()
        .map(|t| (t.verbosity, t.format.clone()));
    assert!(matches!(
        text,
        Some((
            Some(Verbosity::Low),
            Some(TextFormat::JsonSchema {
                strict: Some(true),
                ..
            })
        ))
    ));
    assert_eq!(
        request.include,
        Some(vec![
            Include::ReasoningEncryptedContent,
            Include::WebSearchSources,
            Include::Other("future.include".to_owned()),
        ])
    );
    assert_eq!(request.service_tier, Some(ServiceTier::Priority));
    assert_eq!(request.truncation, Some(Truncation::Auto));
    assert_eq!(request.stream, Some(true));
    assert_eq!(
        request.extra.get("client_metadata"),
        Some(&json!({"origin": "fixture"}))
    );
    Ok(())
}

#[test]
fn request_round_trips_and_tags_bare_messages() -> Result<(), ResponsesError> {
    let request = fixture()?;
    let written = serde_json::to_value(&request)?;
    assert_eq!(written["input"][0]["type"], "message");
    assert_eq!(
        written["input"][9],
        json!({"type": "mcp_approval_response", "approval_request_id": "mcpr_1", "approve": true})
    );
    assert_eq!(
        written["tools"][3],
        json!({"type": "web_search", "search_context_size": "low"})
    );
    assert_eq!(written["client_metadata"], json!({"origin": "fixture"}));
    let again: CreateResponse = serde_json::from_value(written)?;
    assert_eq!(again, request);
    Ok(())
}

#[test]
fn minimal_request_writes_only_what_is_set() -> Result<(), ResponsesError> {
    let mut request = CreateResponse::new(
        ModelId("local-model".to_owned()),
        Input::Text("hi".to_owned()),
    );
    request.tool_choice = Some(ToolChoice::Named(Open::Known(NamedTool::Function {
        name: ToolName("read_file".to_owned()),
    })));
    request.reasoning = Some(openai_responses_api::ReasoningConfig {
        effort: Some(ReasoningEffort::Off),
        summary: None,
    });
    assert_eq!(
        serde_json::to_value(&request)?,
        json!({
            "model": "local-model",
            "input": "hi",
            "tool_choice": {"type": "function", "name": "read_file"},
            "reasoning": {"effort": "none"}
        })
    );
    assert_eq!(serde_json::to_value(ToolChoiceMode::Off)?, json!("none"));
    Ok(())
}

#[test]
fn count_input_tokens_keeps_accepted_fields() -> Result<(), ResponsesError> {
    let request = fixture()?;
    let count = serde_json::to_value(CountInputTokens::from(&request))?;
    let Value::Object(fields) = count else {
        panic!("count body is not an object");
    };
    let mut keys: Vec<&str> = fields.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "input",
            "instructions",
            "model",
            "parallel_tool_calls",
            "reasoning",
            "text",
            "tool_choice",
            "tools",
            "truncation"
        ]
    );
    Ok(())
}

#[test]
fn input_without_type_or_role_is_an_error() {
    let parsed = serde_json::from_value::<CreateResponse>(json!({
        "model": "m",
        "input": [{"call_id": "c", "output": "x"}]
    }));
    let message = match parsed {
        Ok(request) => panic!("parsed an untyped item: {request:?}"),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("type tag"), "{message}");
}
