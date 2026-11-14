use token_sentinel::{
    BudgetConfig, ChatCompletionRequest, ChatMessage, TerminalHud, TokenSentinel,
};

#[tokio::main]
async fn main() {
    println!("================================================================================");
    println!("🛡️  TOKEN-SENTINEL: Context Deduplication & Hallucination Firewall");
    println!("   Guarding: Cursor · Claude Code · Cline · Aider · OpenAI API");
    println!("================================================================================\n");

    let budget = BudgetConfig {
        max_session_cost_usd: 10.00,
        max_request_cost_usd: 2.50,
    };

    // Whitelist legitimate packages present in project manifest
    let known_deps = ["tokio", "serde", "fastapi", "pydantic", "react"];
    let sentinel = TokenSentinel::new(budget, &known_deps);

    // Large code snapshot repeated across multi-turn agent conversation
    let heavy_code_file = r#"```python
# app/core/engine.py (150 lines of database models and service logic)
import os
import sys
from typing import List, Dict, Optional

class DatabaseConnectionPool:
    def __init__(self, host: str, port: int, pool_size: int = 10):
        self.host = host
        self.port = port
        self.pool_size = pool_size
        self.active_connections = []

    def acquire(self):
        print(f"Acquiring connection to {self.host}:{self.port}")
        return True

    def release(self, conn):
        print("Releasing connection back to pool")
        return True

class OrderProcessingPipeline:
    def __init__(self, pool: DatabaseConnectionPool):
        self.pool = pool

    def execute_batch(self, order_ids: List[int]) -> Dict[str, str]:
        print(f"Executing batch for {len(order_ids)} orders")
        return {"status": "success", "processed": len(order_ids)}
```"#;

    println!("[SESSION START] Initializing multi-turn Cursor session with Claude 3.5 Sonnet...\n");

    // ---------------------------------------------------------
    // TURN 1: Initial user prompt with code
    // ---------------------------------------------------------
    println!("👉 TURN 1: User asks AI to review the connection pool.");
    let req1 = ChatCompletionRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert systems engineer.".to_string(),
                name: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Please review this codebase:\n\n{heavy_code_file}\n\nIs the pool connection safe?"),
                name: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(256),
        stream: Some(false),
    };

    let reply1 = "The pool is clean, but make sure to add connection health checks on acquire.";
    let (_, report1) = sentinel
        .process_turn(req1, Some(reply1))
        .await
        .expect("Turn 1 failed");
    print!("{}", TerminalHud::format_turn_report(&report1));

    // ---------------------------------------------------------
    // TURN 2: User asks follow-up. Cursor re-sends entire file!
    // AI hallucinates a non-existent package!
    // ---------------------------------------------------------
    println!("\n👉 TURN 2: Follow-up question. Agent re-sends the exact same file!");
    println!("   AI also hallucinated a package: `import fastapi_limiter_ultra`");

    let req2 = ChatCompletionRequest {
        model: "claude-3-5-sonnet".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an expert systems engineer.".to_string(),
                name: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Reviewing code:\n{heavy_code_file}"),
                name: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: reply1.to_string(),
                name: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Here is the file again:\n{heavy_code_file}\nPlease add a rate limiter!"
                ),
                name: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(256),
        stream: Some(false),
    };

    // Hallucinated code containing fake library
    let reply2 = r#"```python
# Added rate limiter implementation
import os
import fastapi_limiter_ultra  # <-- HALLUCINATED PACKAGE!

def apply_rate_limit():
    limiter = fastapi_limiter_ultra.RateLimiter(rate="100/minute")
    return limiter
```"#;

    let (_, report2) = sentinel
        .process_turn(req2, Some(reply2))
        .await
        .expect("Turn 2 failed");
    print!("{}", TerminalHud::format_turn_report(&report2));

    // ---------------------------------------------------------
    // Cumulative Telemetry & Savings
    // ---------------------------------------------------------
    let final_telemetry = sentinel.get_telemetry();
    print!(
        "{}",
        TerminalHud::format_cumulative_telemetry(&final_telemetry)
    );
}
