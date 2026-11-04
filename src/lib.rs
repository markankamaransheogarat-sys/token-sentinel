//! # token-sentinel
//!
//! Local Context Budget Guardian, Deduplication Proxy & Phantom Package Hallucination Scanner for AI Coding Agents in Rust.

pub mod models;
pub mod optimizer;
pub mod guard;
pub mod proxy;

pub use models::{ModelPrice, PricingRegistry, ChatMessage, ChatCompletionRequest, ChatCompletionResponse, Usage};
pub use optimizer::{ContextDeduplicator, DedupResult, PhantomScanner, PhantomWarning};
pub use guard::{BudgetGuard, BudgetConfig, BudgetError, SentinelMetrics, SentinelTelemetryReport};
pub use proxy::{TokenSentinel, TurnInspectionReport, TerminalHud};
