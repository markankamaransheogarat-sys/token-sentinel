use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct ModelPrice {
    pub prompt_price_per_million: f64,
    pub completion_price_per_million: f64,
}

pub struct PricingRegistry {
    prices: HashMap<String, ModelPrice>,
    default_price: ModelPrice,
}

impl PricingRegistry {
    pub fn new() -> Self {
        let mut prices = HashMap::new();

        // OpenAI Models
        prices.insert(
            "gpt-4o".to_string(),
            ModelPrice { prompt_price_per_million: 2.50, completion_price_per_million: 10.00 },
        );
        prices.insert(
            "gpt-4o-2024-08-06".to_string(),
            ModelPrice { prompt_price_per_million: 2.50, completion_price_per_million: 10.00 },
        );
        prices.insert(
            "gpt-4o-mini".to_string(),
            ModelPrice { prompt_price_per_million: 0.15, completion_price_per_million: 0.60 },
        );
        prices.insert(
            "o1-preview".to_string(),
            ModelPrice { prompt_price_per_million: 15.00, completion_price_per_million: 60.00 },
        );
        prices.insert(
            "o1-mini".to_string(),
            ModelPrice { prompt_price_per_million: 3.00, completion_price_per_million: 12.00 },
        );

        // Anthropic Models
        prices.insert(
            "claude-3-5-sonnet".to_string(),
            ModelPrice { prompt_price_per_million: 3.00, completion_price_per_million: 15.00 },
        );
        prices.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            ModelPrice { prompt_price_per_million: 3.00, completion_price_per_million: 15.00 },
        );
        prices.insert(
            "claude-3-haiku".to_string(),
            ModelPrice { prompt_price_per_million: 0.25, completion_price_per_million: 1.25 },
        );

        // DeepSeek Models
        prices.insert(
            "deepseek-chat".to_string(),
            ModelPrice { prompt_price_per_million: 0.14, completion_price_per_million: 0.28 },
        );
        prices.insert(
            "deepseek-coder".to_string(),
            ModelPrice { prompt_price_per_million: 0.14, completion_price_per_million: 0.28 },
        );

        // Meta Llama Models
        prices.insert(
            "llama-3.1-70b".to_string(),
            ModelPrice { prompt_price_per_million: 0.70, completion_price_per_million: 0.90 },
        );
        prices.insert(
            "llama-3.1-405b".to_string(),
            ModelPrice { prompt_price_per_million: 3.00, completion_price_per_million: 3.00 },
        );

        Self {
            prices,
            default_price: ModelPrice {
                prompt_price_per_million: 2.50,
                completion_price_per_million: 10.00,
            },
        }
    }

    pub fn get_price(&self, model_name: &str) -> ModelPrice {
        for (pattern, price) in &self.prices {
            if model_name.contains(pattern) {
                return *price;
            }
        }
        self.default_price
    }

    pub fn calculate_cost(&self, model: &str, prompt_tokens: usize, completion_tokens: usize) -> f64 {
        let price = self.get_price(model);
        let prompt_cost = (prompt_tokens as f64 / 1_000_000.0) * price.prompt_price_per_million;
        let completion_cost = (completion_tokens as f64 / 1_000_000.0) * price.completion_price_per_million;
        prompt_cost + completion_cost
    }
}
