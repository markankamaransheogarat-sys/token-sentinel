# 🛡️ token-sentinel

**Local Context Budget Guardian, Semantic Deduplication Proxy & Phantom Package Hallucination Firewall for AI Coding Agents**

[![Rust](https://img.shields.io/badge/rust-1.80%2B-blue.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg)](#license)
[![Zero-Dependency](https://img.shields.io/badge/runtime-zero%20C%20libs-brightgreen.svg)](#architecture)
[![Context Savings](https://img.shields.io/badge/context%20savings-30%25--50%25-orange.svg)](#features)

---

## ⚡ The Problem: Why Your AI Agent Bills Are Bleeding

Modern AI coding agents (**Cursor, Claude Code, Cline, Aider, Windsurf**) suffer from two critical architectural inefficiencies:

1. **Massive Context Inflation & Redundancy**:  
   Across multi-turn dialogues, agents repeatedly re-send identical 1,000+ line file snapshots on every minor prompt edit. A user asking *"add a log on line 42"* causes the agent to re-upload 10,000+ tokens of unchanged code, burning hundreds of dollars per developer per month.
2. **Phantom Package Hallucinations & Supply Chain Poisoning**:  
   LLMs frequently invent non-existent package names (`fastapi-limiter-ultra`, `react-virtual-tree-v2`, `crypto-aes-helpers`). Threat actors register these hallucinated package names on PyPI and npm with malware (Dependency Confusion / Slopsquatting attacks), compromising developer machines.

`token-sentinel` is a lightweight, zero-latency local reverse proxy (`http://localhost:8080/v1`) that sits transparently between your IDE and LLM providers.

---

## 🏛️ Architecture Overview

```
 ┌──────────────────────────────────────────────────────────────┐
 │              Developer IDE (Cursor / Claude Code)            │
 └──────────────────────────────┬───────────────────────────────┘
                                │  POST /v1/chat/completions
                                ▼
 ┌──────────────────────────────────────────────────────────────┐
 │                      token-sentinel                          │
 │  ┌────────────────────────────────────────────────────────┐  │
 │  │ 1. Budget Sentinel & Rate Admission                    │  │
 │  │    • Session Hard Limit & Single-Request Cap           │  │
 │  └───────────────────────────┬────────────────────────────┘  │
 │                              │                               │
 │  ┌───────────────────────────▼────────────────────────────┐  │
 │  │ 2. Context Window Semantic Deduplicator                │  │
 │  │    • Rolling Multi-Turn Hash Cache                     │  │
 │  │    • Code Block Compression (-30% to -50% Tokens)      │  │
 │  └───────────────────────────┬────────────────────────────┘  │
 └──────────────────────────────┼───────────────────────────────┘
                                │  Optimized Wire Payload
                                ▼
                 [ Upstream Provider: OpenAI / Anthropic ]
                                │
                                ▼  Streamed Completion
 ┌──────────────────────────────┴───────────────────────────────┐
 │                      token-sentinel                          │
 │  ┌────────────────────────────────────────────────────────┐  │
 │  │ 3. Phantom Package & Hallucination Scanner             │  │
 │  │    • Python / Rust / JS Import Verification            │  │
 │  │    • Stdlib & Manifest Check (Cargo.toml / PyPI)       │  │
 │  └───────────────────────────┬────────────────────────────┘  │
 │                              │                               │
 │  ┌───────────────────────────▼────────────────────────────┐  │
 │  │ 4. Live Terminal Burn HUD                              │  │
 │  │    • Real-time $ Spent vs $ Saved Display              │  │
 │  └────────────────────────────────────────────────────────┘  │
 └──────────────────────────────┬───────────────────────────────┘
                                │  Verified Code & Delta
                                ▼
                 [ IDE Editor Workspace Applied ]
```

---

## 🔬 Core Features

### 1. Rolling Context Deduplication
- Scans prompt message history for repeated file context and code fences (` ```lang ... ``` `).
- Replaces duplicate blocks seen in earlier turns with compact semantic references:  
  `// [TokenSentinel: Omitted 85 duplicate lines]`
- Drops context token consumption by **30% to 50%** on multi-turn refactoring loops without degrading model answer quality.

### 2. Phantom Dependency Firewall
- Scans assistant completion text before applying to disk:
  - **Python**: `import foo`, `from foo.bar import baz`
  - **Rust**: `use foo::bar`, `extern crate foo`
  - **JavaScript/TypeScript**: `import ... from 'foo'`, `require('foo')`
- Cross-references imports against standard libraries and project manifests (`Cargo.toml`, `package.json`, `pyproject.toml`, `requirements.txt`).
- Immediately sounds an alert if the agent invents an unverified third-party package.

### 3. Session Budget Sentinel
- Built-in token pricing matrix for major frontier models:
  - `gpt-4o` ($2.50 / $10.00 per 1M)
  - `claude-3-5-sonnet` ($3.00 / $15.00 per 1M)
  - `deepseek-chat` ($0.14 / $0.28 per 1M)
  - `llama-3.1-70b` ($0.70 / $0.90 per 1M)
- Enforces session budgets (`--max-session-cost 10.00`) to halt runaway agent loops before they drain funds.

---

## 💻 Live HUD & Terminal Telemetry

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 🛡️  TOKEN-SENTINEL: TURN INSPECTION FOR claude-3-5-sonnet                  │
└─────────────────────────────────────────────────────────────────────────────┘
  • Prompt Tokens:      514 -> 297 (Saved 216 tokens, 42.1% compression)
  • Completion Tokens:  60
  • Turn Cost:          $0.00179 (Avoided waste: $0.00065)

  🚨 PHANTOM DEPENDENCY ALERTS DETECTED IN AI CODE:
    1. [PYTHON] Package 'fastapi_limiter_ultra' at: `import fastapi_limiter_ultra`
       -> Package 'fastapi_limiter_ultra' is NOT in project manifests or standard library. 
          Verify it exists to prevent dependency confusion attack or broken build.
───────────────────────────────────────────────────────────────────────────────

===============================================================================
📊 TOKEN-SENTINEL CUMULATIVE SESSION TELEMETRY
===============================================================================
  Total AI Turns:               2
  Total Prompt Tokens:          556
  Total Completion Tokens:      80
  Total Tokens Saved:           216 (28.0% net context reduction)
  Total Spent:                  $0.0029 USD
  Total Saved:                  $0.0006 USD 💰
  Phantom Warnings Intercepted: 1
===============================================================================
```

---

## 🚀 Quick Start

### Build & Run
```bash
cargo run --release
```

### Run Tests
```bash
cargo test
```

### Configure in Cursor / Claude Code
In Cursor Settings $\to$ **Models** $\to$ **OpenAI Base URL**:
```
http://localhost:8080/v1
```

---

## 📜 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
