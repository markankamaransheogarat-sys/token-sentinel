use token_sentinel::{BudgetConfig, BudgetError, BudgetGuard};

#[test]
fn test_budget_guard_limits() {
    let config = BudgetConfig {
        max_session_cost_usd: 1.00,
        max_request_cost_usd: 0.50,
    };

    let mut guard = BudgetGuard::new(config);

    // Request within limits
    assert!(guard.check_admission(0.20).is_ok());
    guard.record_spend(0.20);
    assert!((guard.total_spent() - 0.20).abs() < 1e-6);

    // Request exceeding single request limit
    let err_single = guard.check_admission(0.60);
    assert!(matches!(
        err_single,
        Err(BudgetError::RequestBudgetExceeded { .. })
    ));

    // Spend up to 0.90
    guard.record_spend(0.70);
    assert!((guard.total_spent() - 0.90).abs() < 1e-6);

    // Next 0.20 request exceeds session limit ($1.10 > $1.00)
    let err_session = guard.check_admission(0.20);
    assert!(matches!(
        err_session,
        Err(BudgetError::SessionBudgetExceeded { .. })
    ));
}
