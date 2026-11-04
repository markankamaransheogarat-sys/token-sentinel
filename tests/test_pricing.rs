use token_sentinel::PricingRegistry;

#[test]
fn test_pricing_calculations() {
    let registry = PricingRegistry::new();

    // 1M prompt + 1M completion for gpt-4o -> $2.50 + $10.00 = $12.50
    let cost = registry.calculate_cost("gpt-4o", 1_000_000, 1_000_000);
    assert!((cost - 12.50).abs() < 1e-6);

    // 10k prompt + 1k completion for claude-3-5-sonnet -> (0.01 * 3.0) + (0.001 * 15.0) = 0.03 + 0.015 = 0.045
    let cost_claude = registry.calculate_cost("claude-3-5-sonnet-20241022", 10_000, 1_000);
    assert!((cost_claude - 0.045).abs() < 1e-6);

    // DeepSeek coder low cost test
    let cost_deepseek = registry.calculate_cost("deepseek-chat", 1_000_000, 1_000_000);
    assert!((cost_deepseek - 0.42).abs() < 1e-6);
}
