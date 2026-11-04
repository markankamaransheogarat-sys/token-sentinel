use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use crate::models::protocol::ChatMessage;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DedupResult {
    pub original_estimated_tokens: usize,
    pub optimized_estimated_tokens: usize,
    pub tokens_saved: usize,
    pub savings_ratio: f64,
    pub blocks_deduplicated: usize,
}

pub struct ContextDeduplicator {
    min_block_lines: usize,
}

impl ContextDeduplicator {
    pub fn new(min_block_lines: usize) -> Self {
        Self { min_block_lines }
    }

    pub fn with_defaults() -> Self {
        Self::new(8) // Deduplicate code blocks with 8+ lines
    }

    /// Fast token estimation (standard heuristic: ~3.8 chars per token for code/text)
    pub fn estimate_tokens(text: &str) -> usize {
        let chars = text.chars().count();
        if chars == 0 {
            0
        } else {
            ((chars as f64) / 3.8).ceil() as usize
        }
    }

    fn hash_block(lines: &[&str]) -> u64 {
        let mut hasher = DefaultHasher::new();
        for line in lines {
            line.trim().hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Deduplicates repetitive code context across chat messages
    pub fn optimize_messages(&self, messages: &[ChatMessage]) -> (Vec<ChatMessage>, DedupResult) {
        let mut seen_blocks: HashSet<u64> = HashSet::new();
        let mut optimized_messages = Vec::with_capacity(messages.len());

        let mut total_original_chars = 0;
        let mut total_optimized_chars = 0;
        let mut total_blocks_deduped = 0;

        for msg in messages {
            total_original_chars += msg.content.chars().count();

            // Extract fenced code blocks: ```lang ... ```
            let mut new_content = String::with_capacity(msg.content.len());
            let lines: Vec<&str> = msg.content.lines().collect();

            let mut i = 0;
            while i < lines.len() {
                if lines[i].trim_start().starts_with("```") {
                    let code_start = i;
                    let fence_header = lines[i];
                    i += 1;

                    let mut code_lines = Vec::new();
                    while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                        code_lines.push(lines[i]);
                        i += 1;
                    }

                    let code_end = i;
                    let fence_footer = if code_end < lines.len() { lines[code_end] } else { "```" };

                    if code_lines.len() >= self.min_block_lines {
                        let block_hash = Self::hash_block(&code_lines);
                        if seen_blocks.contains(&block_hash) {
                            // Deduplicate block!
                            total_blocks_deduped += 1;
                            new_content.push_str(fence_header);
                            new_content.push('\n');
                            new_content.push_str(&format!(
                                "// [TokenSentinel: Omitted {} duplicate lines]\n",
                                code_lines.len()
                            ));
                            new_content.push_str(fence_footer);
                            new_content.push('\n');

                            if code_end < lines.len() {
                                i += 1;
                            }
                            continue;
                        } else {
                            seen_blocks.insert(block_hash);
                        }
                    }

                    // Not deduplicated, append original block
                    for l in &lines[code_start..=code_end.min(lines.len() - 1)] {
                        new_content.push_str(l);
                        new_content.push('\n');
                    }
                    if code_end < lines.len() {
                        i += 1;
                    }
                } else {
                    new_content.push_str(lines[i]);
                    new_content.push('\n');
                    i += 1;
                }
            }

            total_optimized_chars += new_content.chars().count();

            optimized_messages.push(ChatMessage {
                role: msg.role.clone(),
                content: new_content,
                name: msg.name.clone(),
            });
        }

        let orig_tokens = Self::estimate_tokens_from_chars(total_original_chars);
        let opt_tokens = Self::estimate_tokens_from_chars(total_optimized_chars);
        let saved = orig_tokens.saturating_sub(opt_tokens);
        let ratio = if orig_tokens > 0 {
            (saved as f64) / (orig_tokens as f64)
        } else {
            0.0
        };

        let result = DedupResult {
            original_estimated_tokens: orig_tokens,
            optimized_estimated_tokens: opt_tokens,
            tokens_saved: saved,
            savings_ratio: ratio,
            blocks_deduplicated: total_blocks_deduped,
        };

        (optimized_messages, result)
    }

    fn estimate_tokens_from_chars(chars: usize) -> usize {
        if chars == 0 {
            0
        } else {
            ((chars as f64) / 3.8).ceil() as usize
        }
    }
}
