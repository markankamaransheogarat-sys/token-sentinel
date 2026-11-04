#[derive(Debug, Clone, serde::Serialize)]
pub struct BudgetConfig {
    pub max_session_cost_usd: f64,
    pub max_request_cost_usd: f64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_session_cost_usd: 15.00,
            max_request_cost_usd: 2.00,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BudgetError {
    SessionBudgetExceeded { current_spent: f64, limit: f64 },
    RequestBudgetExceeded { estimated_cost: f64, limit: f64 },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionBudgetExceeded { current_spent, limit } => {
                write!(f, "Session budget limit exceeded! Spent: ${current_spent:.4}, Limit: ${limit:.2}")
            }
            Self::RequestBudgetExceeded { estimated_cost, limit } => {
                write!(f, "Single request estimated cost (${estimated_cost:.4}) exceeds limit (${limit:.2})")
            }
        }
    }
}

impl std::error::Error for BudgetError {}

pub struct BudgetGuard {
    config: BudgetConfig,
    cumulative_spent: f64,
}

impl BudgetGuard {
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            cumulative_spent: 0.0,
        }
    }

    pub fn check_admission(&self, estimated_req_cost: f64) -> Result<(), BudgetError> {
        if estimated_req_cost > self.config.max_request_cost_usd {
            return Err(BudgetError::RequestBudgetExceeded {
                estimated_cost: estimated_req_cost,
                limit: self.config.max_request_cost_usd,
            });
        }

        if (self.cumulative_spent + estimated_req_cost) > self.config.max_session_cost_usd {
            return Err(BudgetError::SessionBudgetExceeded {
                current_spent: self.cumulative_spent,
                limit: self.config.max_session_cost_usd,
            });
        }

        Ok(())
    }

    pub fn record_spend(&mut self, cost_usd: f64) {
        self.cumulative_spent += cost_usd;
    }

    pub fn remaining_budget(&self) -> f64 {
        (self.config.max_session_cost_usd - self.cumulative_spent).max(0.0)
    }

    pub fn total_spent(&self) -> f64 {
        self.cumulative_spent
    }
}
