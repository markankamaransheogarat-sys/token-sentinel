use std::sync::Arc;
use tokio::sync::Mutex;
use crate::models::{ChatCompletionRequest, ChatCompletionResponse, ChatChoice, ChatMessage, Usage, PricingRegistry};
use crate::optimizer::{ContextDeduplicator, PhantomScanner, PhantomWarning};
use crate::guard::{BudgetGuard, BudgetConfig, BudgetError, SentinelMetrics, SentinelTelemetryReport};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TurnInspectionReport {
    pub model: String,
    pub original_prompt_tokens: usize,
    pub optimized_prompt_tokens: usize,
    pub completion_tokens: usize,
    pub tokens_saved: usize,
    pub savings_percentage: f64,
    pub cost_spent_usd: f64,
    pub cost_saved_usd: f64,
    pub phantom_warnings: Vec<PhantomWarning>,
}

pub struct TokenSentinel {
    pricing: PricingRegistry,
    deduplicator: ContextDeduplicator,
    phantom_scanner: PhantomScanner,
    budget_guard: Arc<Mutex<BudgetGuard>>,
    metrics: Arc<SentinelMetrics>,
}

impl TokenSentinel {
    pub fn new(budget_config: BudgetConfig, known_dependencies: &[&str]) -> Self {
        Self {
            pricing: PricingRegistry::new(),
            deduplicator: ContextDeduplicator::with_defaults(),
            phantom_scanner: PhantomScanner::with_known_dependencies(known_dependencies),
            budget_guard: Arc::new(Mutex::new(BudgetGuard::new(budget_config))),
            metrics: Arc::new(SentinelMetrics::new()),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(BudgetConfig::default(), &[])
    }

    /// Process an incoming chat completion request:
    /// 1. Budget check
    /// 2. Deduplicate messages
    /// 3. Execute mock/upstream completion
    /// 4. Scan assistant completion for phantom dependencies
    /// 5. Record telemetry
    pub async fn process_turn(
        &self,
        mut req: ChatCompletionRequest,
        mock_assistant_reply: Option<&str>,
    ) -> Result<(ChatCompletionResponse, TurnInspectionReport), BudgetError> {
        let model = req.model.clone();

        // 1. Initial Token Estimation & Budget Pre-Check
        let raw_prompt_tokens: usize = req.messages.iter()
            .map(|m| ContextDeduplicator::estimate_tokens(&m.content))
            .sum();

        let estimated_cost = self.pricing.calculate_cost(&model, raw_prompt_tokens, 256);
        {
            let guard = self.budget_guard.lock().await;
            guard.check_admission(estimated_cost)?;
        }

        // 2. Context Window Deduplication
        let (opt_messages, dedup_result) = self.deduplicator.optimize_messages(&req.messages);
        req.messages = opt_messages;

        let optimized_prompt_tokens = dedup_result.optimized_estimated_tokens;
        let tokens_saved = dedup_result.tokens_saved;

        // 3. Generate or retrieve completion
        let reply_text = mock_assistant_reply.unwrap_or(
            "// Auto-generated implementation\nuse tokio::net::TcpListener;\n\npub async fn run() {\n    println!(\"Server running\");\n}\n"
        );

        let completion_tokens = ContextDeduplicator::estimate_tokens(reply_text);

        // 4. Scan for Phantom / Hallucinated Dependencies
        let phantom_warnings = self.phantom_scanner.scan_code(reply_text);

        // 5. Cost Calculation & Telemetry
        let actual_spent_usd = self.pricing.calculate_cost(&model, optimized_prompt_tokens, completion_tokens);
        let actual_saved_usd = self.pricing.calculate_cost(&model, tokens_saved, 0);

        {
            let mut guard = self.budget_guard.lock().await;
            guard.record_spend(actual_spent_usd);
        }

        self.metrics.record_turn(
            optimized_prompt_tokens,
            completion_tokens,
            tokens_saved,
            actual_spent_usd,
            actual_saved_usd,
            phantom_warnings.len(),
        );

        let response = ChatCompletionResponse {
            id: format!("sentinel-cmpl-{}", 10001),
            object: "chat.completion".to_string(),
            created: 1730000000,
            model: model.clone(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: reply_text.to_string(),
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Usage {
                prompt_tokens: optimized_prompt_tokens,
                completion_tokens,
                total_tokens: optimized_prompt_tokens + completion_tokens,
            },
        };

        let report = TurnInspectionReport {
            model,
            original_prompt_tokens: raw_prompt_tokens,
            optimized_prompt_tokens,
            completion_tokens,
            tokens_saved,
            savings_percentage: dedup_result.savings_ratio * 100.0,
            cost_spent_usd: actual_spent_usd,
            cost_saved_usd: actual_saved_usd,
            phantom_warnings,
        };

        Ok((response, report))
    }

    pub fn get_telemetry(&self) -> SentinelTelemetryReport {
        self.metrics.generate_report()
    }
}
