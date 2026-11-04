use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SentinelTelemetryReport {
    pub total_requests: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens_saved: u64,
    pub tokens_savings_ratio: f64,
    pub total_spent_usd: f64,
    pub total_saved_usd: f64,
    pub phantom_warnings_count: u64,
}

pub struct SentinelMetrics {
    total_requests: AtomicU64,
    total_prompt_tokens: AtomicU64,
    total_completion_tokens: AtomicU64,
    total_tokens_saved: AtomicU64,
    phantom_warnings: AtomicU64,
    cost_spent: Mutex<f64>,
    cost_saved: Mutex<f64>,
}

impl SentinelMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_prompt_tokens: AtomicU64::new(0),
            total_completion_tokens: AtomicU64::new(0),
            total_tokens_saved: AtomicU64::new(0),
            phantom_warnings: AtomicU64::new(0),
            cost_spent: Mutex::new(0.0),
            cost_saved: Mutex::new(0.0),
        }
    }

    pub fn record_turn(
        &self,
        prompt_tokens: usize,
        completion_tokens: usize,
        saved_tokens: usize,
        spent_usd: f64,
        saved_usd: f64,
        warnings_count: usize,
    ) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_prompt_tokens.fetch_add(prompt_tokens as u64, Ordering::Relaxed);
        self.total_completion_tokens.fetch_add(completion_tokens as u64, Ordering::Relaxed);
        self.total_tokens_saved.fetch_add(saved_tokens as u64, Ordering::Relaxed);
        self.phantom_warnings.fetch_add(warnings_count as u64, Ordering::Relaxed);

        if let Ok(mut spent) = self.cost_spent.lock() {
            *spent += spent_usd;
        }
        if let Ok(mut saved) = self.cost_saved.lock() {
            *saved += saved_usd;
        }
    }

    pub fn generate_report(&self) -> SentinelTelemetryReport {
        let total_prompt = self.total_prompt_tokens.load(Ordering::Relaxed);
        let total_saved = self.total_tokens_saved.load(Ordering::Relaxed);

        let total_raw_prompt = total_prompt + total_saved;
        let savings_ratio = if total_raw_prompt > 0 {
            total_saved as f64 / total_raw_prompt as f64
        } else {
            0.0
        };

        let spent = self.cost_spent.lock().map(|g| *g).unwrap_or(0.0);
        let saved = self.cost_saved.lock().map(|g| *g).unwrap_or(0.0);

        SentinelTelemetryReport {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_prompt_tokens: total_prompt,
            total_completion_tokens: self.total_completion_tokens.load(Ordering::Relaxed),
            total_tokens_saved: total_saved,
            tokens_savings_ratio: savings_ratio,
            total_spent_usd: spent,
            total_saved_usd: saved,
            phantom_warnings_count: self.phantom_warnings.load(Ordering::Relaxed),
        }
    }
}
