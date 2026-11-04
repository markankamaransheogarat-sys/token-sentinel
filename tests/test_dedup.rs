use token_sentinel::{ChatMessage, ContextDeduplicator};

#[test]
fn test_context_window_deduplication() {
    let dedup = ContextDeduplicator::new(5);

    let code_block = r#"```rust
pub struct ServerEngine {
    host: String,
    port: u16,
    max_connections: usize,
    buffer_capacity: usize,
    is_running: bool,
}

impl ServerEngine {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            max_connections: 1024,
            buffer_capacity: 65536,
            is_running: false,
        }
    }

    pub fn start(&mut self) {
        println!("Server starting at {}:{}", self.host, self.port);
        self.is_running = true;
    }
}
```"#;

    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: format!("Here is my code:\n{code_block}"),
            name: None,
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Looks good!".to_string(),
            name: None,
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("Re-sending file:\n{code_block}\nPlease refactor!"),
            name: None,
        },
    ];

    let (optimized, result) = dedup.optimize_messages(&messages);

    assert_eq!(result.blocks_deduplicated, 1);
    assert!(result.tokens_saved > 0);
    assert!(result.savings_ratio > 0.0);

    // Turn 3 should have the omission comment
    assert!(optimized[2].content.contains("TokenSentinel: Omitted"));
}
