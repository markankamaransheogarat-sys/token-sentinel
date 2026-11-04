use super::server::TurnInspectionReport;
use crate::guard::SentinelTelemetryReport;

pub struct TerminalHud;

impl TerminalHud {
    pub fn format_turn_report(report: &TurnInspectionReport) -> String {
        let mut out = String::new();
        out.push_str("\n┌─────────────────────────────────────────────────────────────────────────────┐\n");
        out.push_str(&format!("│ 🛡️  TOKEN-SENTINEL: TURN INSPECTION FOR {:<34} │\n", report.model));
        out.push_str("└─────────────────────────────────────────────────────────────────────────────┘\n");

        out.push_str(&format!(
            "  • Prompt Tokens:      {} -> {} (Saved {} tokens, {:.1}% compression)\n",
            report.original_prompt_tokens,
            report.optimized_prompt_tokens,
            report.tokens_saved,
            report.savings_percentage
        ));
        out.push_str(&format!("  • Completion Tokens:  {}\n", report.completion_tokens));
        out.push_str(&format!(
            "  • Turn Cost:          ${:.5} (Avoided waste: ${:.5})\n",
            report.cost_spent_usd, report.cost_saved_usd
        ));

        if !report.phantom_warnings.is_empty() {
            out.push_str("\n  🚨 PHANTOM DEPENDENCY ALERTS DETECTED IN AI CODE:\n");
            for (idx, w) in report.phantom_warnings.iter().enumerate() {
                out.push_str(&format!(
                    "    {}. [{}] Package '{}' at: `{}`\n",
                    idx + 1,
                    w.language.to_uppercase(),
                    w.package_name,
                    w.statement
                ));
                out.push_str(&format!("       -> {}\n", w.explanation));
            }
        } else {
            out.push_str("  ✅ Hallucination Check: Clean (All imports verified against stdlib/manifests)\n");
        }

        out.push_str("───────────────────────────────────────────────────────────────────────────────\n");
        out
    }

    pub fn format_cumulative_telemetry(report: &SentinelTelemetryReport) -> String {
        let mut out = String::new();
        out.push_str("\n===============================================================================\n");
        out.push_str("📊 TOKEN-SENTINEL CUMULATIVE SESSION TELEMETRY\n");
        out.push_str("===============================================================================\n");
        out.push_str(&format!("  Total AI Turns:           {}\n", report.total_requests));
        out.push_str(&format!("  Total Prompt Tokens:      {}\n", report.total_prompt_tokens));
        out.push_str(&format!("  Total Completion Tokens:  {}\n", report.total_completion_tokens));
        out.push_str(&format!(
            "  Total Tokens Saved:       {} ({:.1}% net context reduction)\n",
            report.total_tokens_saved,
            report.tokens_savings_ratio * 100.0
        ));
        out.push_str(&format!("  Total Spent:              ${:.4} USD\n", report.total_spent_usd));
        out.push_str(&format!("  Total Saved:              ${:.4} USD 💰\n", report.total_saved_usd));
        out.push_str(&format!("  Phantom Warnings Intercepted: {}\n", report.phantom_warnings_count));
        out.push_str("===============================================================================\n");
        out
    }
}
