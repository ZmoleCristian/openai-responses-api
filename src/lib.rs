#![deny(unused_must_use)]
#![deny(for_loops_over_fallibles)]
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_assignments)]

mod client;
mod config;
mod content;
mod error;
mod ids;
mod item;
mod open;
mod request;
mod response;
mod retry_after;
mod stream;

pub use client::Client;
pub use config::{Auth, Config, OPENAI_BASE_URL};
pub use content::{
    Annotation, AnnotationKind, Content, ContentPart, FileSource, ImageDetail, ImageSource,
    InputFile, InputImage, LogProb, OutputText, Part, Role, TopLogProb,
};
pub use error::ResponsesError;
pub use ids::{
    Arguments, CallId, EncryptedContent, FileData, FileId, FileName, ItemId, ModelId,
    OrganizationId, Param, ProjectId, PromptCacheKey, ResponseId, SafetyIdentifier, SchemaName,
    Secret, ShellUser, ToolName,
};
pub use item::{
    CompactionItem, CustomToolCall, CustomToolCallOutput, FunctionCall, FunctionCallOutput,
    InputItem, Item, ItemStatus, LocalShellAction, LocalShellCall, MessageItem, MessagePhase,
    OutputItem, ReasoningItem, SearchAction, SearchSource, ShellAction, WebSearchAction,
    WebSearchCall,
};
pub use open::{Kinds, Open};
pub use request::{
    CountInputTokens, CreateResponse, CustomFormat, CustomTool, FormatKind, FunctionTool,
    GrammarSyntax, Include, Input, NamedTool, ReasoningConfig, ReasoningEffort, ReasoningSummary,
    ServiceTier, StreamOptions, TextConfig, TextFormat, Tool, ToolChoice, ToolChoiceMode, ToolKind,
    Truncation, Verbosity,
};
pub use response::{
    ApiError, ApiErrorBody, ApiErrorKind, Deleted, ErrorCode, IncompleteDetails, IncompleteReason,
    InputTokens, InputTokensDetails, OutputTokensDetails, Response, ResponseError, ResponseStatus,
    Usage,
};
pub use retry_after::RetryAfter;
pub use stream::{
    AnnotationAdded, ArgumentsDone, ChatGptModerationMetadata, ContentPartEvent, Event, InputDone,
    ItemDelta, ItemProgress, Lifecycle, MetadataEvent, ModerationCheck, ModerationCheckOutput,
    ModerationSection, OutputItemEvent, PartDelta, RefusalDone, ResponseMetadata, StreamError,
    StreamEvent, SummaryDelta, SummaryDone, SummaryPartEvent, TextDone, events,
};
pub use url::Url;
