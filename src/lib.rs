//! # token-sentinel
//!
//! Local Context Budget Guardian, Deduplication Proxy & Phantom Package Hallucination Scanner for AI Coding Agents in Rust.

pub mod guard;
pub mod models;
pub mod optimizer;
pub mod proxy;

pub use guard::{BudgetConfig, BudgetError, BudgetGuard, SentinelMetrics, SentinelTelemetryReport};
pub use models::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ModelPrice, PricingRegistry, Usage,
};
pub use optimizer::{ContextDeduplicator, DedupResult, PhantomScanner, PhantomWarning};
pub use proxy::{TerminalHud, TokenSentinel, TurnInspectionReport};
