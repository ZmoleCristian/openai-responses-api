use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

use crate::content::{Content, ContentPart, Role};
use crate::error::ResponsesError;
use crate::ids::{Arguments, CallId, EncryptedContent, ItemId, ShellUser, ToolName};
use crate::open::{Kinds, Open};

pub type InputItem = Open<Item>;
pub type OutputItem = Open<Item>;
pub type WebSearchAction = Open<SearchAction>;
pub type ShellAction = Open<LocalShellAction>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    Message(MessageItem),
    FunctionCall(FunctionCall),
    FunctionCallOutput(FunctionCallOutput),
    CustomToolCall(CustomToolCall),
    CustomToolCallOutput(CustomToolCallOutput),
    Reasoning(ReasoningItem),
    WebSearchCall(WebSearchCall),
    LocalShellCall(LocalShellCall),
    Compaction(CompactionItem),
    ItemReference { id: ItemId },
}

impl Kinds for Item {
    const KINDS: &'static [&'static str] = &[
        "message",
        "function_call",
        "function_call_output",
        "custom_tool_call",
        "custom_tool_call_output",
        "reasoning",
        "web_search_call",
        "local_shell_call",
        "compaction",
        "item_reference",
    ];

    fn untyped(value: Value) -> Result<Value, ResponsesError> {
        let Value::Object(mut map) = value else {
            return Err(ResponsesError::Untyped(value));
        };
        if !map.contains_key("role") {
            return Err(ResponsesError::Untyped(Value::Object(map)));
        }
        map.insert("type".to_owned(), Value::String("message".to_owned()));
        Ok(Value::Object(map))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    InProgress,
    Searching,
    Completed,
    Incomplete,
    Failed,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessagePhase {
    Commentary,
    FinalAnswer,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub role: Role,
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<MessagePhase>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl MessageItem {
    pub fn new(role: Role, content: Content) -> MessageItem {
        MessageItem {
            id: None,
            role,
            content,
            status: None,
            phase: None,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub call_id: CallId,
    pub name: ToolName,
    pub arguments: Arguments,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub call_id: CallId,
    pub output: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomToolCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub call_id: CallId,
    pub name: ToolName,
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomToolCallOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub call_id: CallId,
    pub output: Content,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    #[serde(default)]
    pub summary: Vec<ContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<EncryptedContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSearchCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<WebSearchAction>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchAction {
    Search {
        #[serde(skip_serializing_if = "Option::is_none")]
        query: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sources: Option<Vec<Open<SearchSource>>>,
    },
    OpenPage {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<Url>,
    },
    Find {
        url: Url,
        pattern: String,
    },
}

impl Kinds for SearchAction {
    const KINDS: &'static [&'static str] = &["search", "open_page", "find"];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchSource {
    Url { url: Url },
}

impl Kinds for SearchSource {
    const KINDS: &'static [&'static str] = &["url"];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalShellCall {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub call_id: CallId,
    pub action: ShellAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ItemStatus>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalShellAction {
    Exec {
        command: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        working_directory: Option<PathBuf>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<BTreeMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user: Option<ShellUser>,
    },
}

impl Kinds for LocalShellAction {
    const KINDS: &'static [&'static str] = &["exec"];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompactionItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<ItemId>,
    pub encrypted_content: EncryptedContent,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
