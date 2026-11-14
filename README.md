# openai-responses-api

A typed Rust client for the OpenAI Responses API (`POST /v1/responses` and
its siblings). It works against OpenAI and against any server that speaks the
same wire format, such as vLLM or a gateway in front of it. This is the
protocol Codex speaks.

It is a library only: input and output items, request and response types, an
async HTTP client on reqwest with rustls, and the SSE stream as a typed event
stream.

## Usage

```rust
use openai_responses_api::{
    Auth, Client, Config, Content, CreateResponse, Event, Include, Input, Item, MessageItem,
    ModelId, Open, ReasoningConfig, ReasoningEffort, ReasoningSummary, ResponsesError, Role,
    Secret, Url,
};
use futures_util::StreamExt;

async fn run(key: String) -> Result<(), ResponsesError> {
    let mut config = Config::new(Auth::Bearer(Secret(key)))?;
    config.base_url = Url::parse("http://10.0.0.5:8000/v1")?;
    let client = Client::new(config)?;

    let mut request = CreateResponse::new(
        ModelId("gpt-5.5".to_owned()),
        Input::Items(vec![Open::Known(Item::Message(MessageItem::new(
            Role::User,
            Content::Text("hi".to_owned()),
        )))]),
    );
    request.reasoning = Some(ReasoningConfig {
        effort: Some(ReasoningEffort::High),
        summary: Some(ReasoningSummary::Auto),
    });
    request.include = Some(vec![Include::ReasoningEncryptedContent]);
    request.store = Some(false);

    let reply = client.create(&request).await?;
    println!("{}", reply.output_text());

    let mut events = Box::pin(client.stream(&request).await?);
    while let Some(event) = events.next().await {
        if let Open::Known(Event::OutputTextDelta(delta)) = event? {
            print!("{}", delta.delta);
        }
    }
    Ok(())
}
```

## Configuration

`Config::new(auth)` fills the defaults: base URL `https://api.openai.com/v1`,
no organization or project, no extra headers, a fresh reqwest client. Every
field is public:

- `base_url`: a `Url`; the client appends `responses` to its path, so
  `http://host:8000/v1` and `http://host/gateway/v1/` both work.
- `auth`: `Auth::Bearer` sends `Authorization: Bearer`, `Auth::ApiKey` sends
  `api-key` (Azure style). Both are marked sensitive in the header map.
- `organization`, `project`: `OpenAI-Organization` and `OpenAI-Project`.
- `headers`: extra headers on every request.
- `http`: your own `reqwest::Client` (timeouts, proxies, pools).

## Coverage

- Client: `create`, `stream`, `retrieve`, `cancel`, `delete` and
  `input_tokens`. `create` and `stream` set `stream` themselves.
- Items, shared by input and output: `message` (roles user, assistant,
  system, developer; `phase`; text or parts), `function_call` and
  `function_call_output`, `custom_tool_call` and `custom_tool_call_output`,
  `reasoning` (summary, content, `encrypted_content`), `web_search_call`
  (search, open_page, find actions and sources), `local_shell_call` (exec
  action), `compaction`, `item_reference`. An input item with a `role` and no
  `type` is read as a message, as the API allows.
- Content parts: `input_text`, `input_image` (URL or file id, `detail`),
  `input_file` (file id, URL or data), `output_text` (annotations, logprobs),
  `refusal`, `summary_text`, `reasoning_text`.
- Requests: `instructions`, tools (`function` with `strict`, `custom` with a
  grammar or text format, `local_shell`, `web_search` with its options kept
  as a map), `tool_choice` (none, auto, required, or a named function or
  custom tool), `parallel_tool_calls`, `reasoning` (effort and summary),
  `text` (verbosity, JSON schema or JSON object format), `include`,
  `max_output_tokens`, `max_tool_calls`, sampling, `top_logprobs`, `store`,
  `previous_response_id`, `metadata`, `prompt_cache_key`,
  `safety_identifier`, `service_tier`, `truncation`, `background`,
  `stream_options`. Any other field goes in `extra`, which is flattened into
  the body. `CountInputTokens::from(&request)` keeps the fields the
  `input_tokens` endpoint accepts.
- Responses: status, output, `output_text()`, usage with cached and
  reasoning token details, error, incomplete details, service tier.
- Stream: lifecycle events (`created`, `queued`, `in_progress`, `completed`,
  `failed`, `incomplete`), output items and content parts added and done,
  output text delta, done and annotations, refusal, function call arguments,
  custom tool input, reasoning summary parts and text, reasoning text, web
  search progress, and `error`. `events(bytes)` parses any SSE byte stream,
  so it also works on recorded bodies.

Chat Completions is not covered.

## Unknown data

Servers add fields, item types and event types; this crate keeps them instead
of failing or dropping them.

- Unknown fields land in an `extra` map on requests, responses, items, tools
  and usage.
- Items, content parts, annotations, tools, tool choices, search actions and
  stream events are `Open<K>`: `Open::Known(K)` for a type this crate knows,
  `Open::Unknown(Value)` for any other `type` tag, with the raw JSON kept. A
  known type with a malformed body is an error, not `Unknown`, so a broken
  item from a server is never silently reclassified. An object without a
  `type` tag is an error too.
- A `[DONE]` frame, which some compatible servers send, comes through as
  `Open::Unknown(Value::String("[DONE]"))`.
- Open string sets (statuses, error codes and kinds, reasoning effort and
  summary, include values, service tier, message phase) are enums with an
  `Other(String)` variant. Error codes also take the numeric form some
  servers send.

## Types over strings

Identifiers and tags are types, not `String`: `ModelId`, `ResponseId`,
`ItemId`, `CallId`, `ToolName`, `FileId`, `FileName`, `FileData`,
`SchemaName`, `EncryptedContent`, `PromptCacheKey`, `SafetyIdentifier`,
`OrganizationId`, `ProjectId`, `Param`, `ShellUser`, `Secret`, `Role`, and
the enums above. Function call arguments are `Arguments`, with
`parse::<T>()` to read them into your own type. `String` is left only for
free text: message and reasoning text, instructions, descriptions, custom
tool input, grammar definitions, search queries, shell argv and stream
deltas.

## Errors

One enum, `ResponsesError`, written by hand: transport (`Http`), header
(`Header`), JSON (`Json`), SSE framing (`Sse`), an API error body with its
status (`Api`), a non-success status whose body did not parse (`Status`,
which keeps the body and the parse error), an object without a type tag
(`Untyped`), and base URL problems (`BaseUrl`, `CannotBeABase`).

## Code law

The crate is judged by `lawkeeper` with the default `law.toml`: no comments in
`src/`, no silent error handling, no erased error types or `Result` aliases,
constants and defaults only in `src/config.rs`. Tests live in `tests/` and run
against recorded JSON and SSE fixtures in `tests/fixtures/`, with no network.

```sh
cargo test && lawkeeper
```

## Contributing

The code is judged by [lawkeeper](https://github.com/ZmoleCristian/lawkeeper),
also on crates.io:

```sh
cargo install lawkeeper
```

A pull request must pass `lawkeeper` with this repository's `law.toml`, and
`cargo test`.

## License

0BSD. Written from the public Responses API wire format.
