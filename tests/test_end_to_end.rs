use token_sentinel::{BudgetConfig, ChatCompletionRequest, ChatMessage, TokenSentinel};

#[tokio::test]
async fn test_end_to_end_sentinel_workflow() {
    let budget = BudgetConfig {
        max_session_cost_usd: 5.00,
        max_request_cost_usd: 1.00,
    };
    let sentinel = TokenSentinel::new(budget, &["fastapi"]);

    let repetitive_code = "```python\ndef handler():\n    a = 10\n    b = 20\n    c = 30\n    d = 40\n    e = 50\n    f = 60\n    g = 70\n    h = 80\n    return a+b+c+d+e+f+g+h\n```";

    let req = ChatCompletionRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: format!("Check this code:\n{repetitive_code}"),
                name: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Got it.".to_string(),
                name: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Again:\n{repetitive_code}\nPlease optimize!"),
                name: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: Some(false),
    };

    let reply = "```python\nimport non_existent_super_lib\n```";

    let (resp, report) = sentinel.process_turn(req, Some(reply)).await.unwrap();

    assert_eq!(resp.model, "gpt-4o");
    assert!(report.tokens_saved > 0);
    assert_eq!(report.phantom_warnings.len(), 1);
    assert_eq!(
        report.phantom_warnings[0].package_name,
        "non_existent_super_lib"
    );

    let tele = sentinel.get_telemetry();
    assert_eq!(tele.total_requests, 1);
    assert!(tele.total_tokens_saved > 0);
    assert!(tele.total_spent_usd > 0.0);
}
