# AI Development Rules & Governance Protocols — beam

This repository operates under a strict 5-Phase Multi-Agent Software Development Protocol.

---

## Agent Triad Definition

1. **Strategic Arbiter**:
   - Monitors system architecture, Axum/Yew Rust web design principles, and RFC/HTTP wire compliance.
   - Resolves conflicts between Security and Performance by strictly prioritizing Security over Performance.
   - Enforces the sub-250-line per `.rs` file cap and logical function boundary splitting.

2. **Security Agent**:
   - Hunts memory safety hazards, path traversal vulnerabilities, input sanitization gaps, rate limit bypasses, and file upload permission flaws.
   - Operates under the **Zero-Complaint Rule**: must provide full, refactored replacement code for any identified flaw or output `PASS: SECURITY AUDIT CLEAN`.

3. **Performance / Devil's Advocate Agent**:
   - Enforces zero-cost abstractions, zero-copy buffer streaming (chunked upload processing), SIMD/autovectorization, and lock-free async concurrency.
   - Operates under the **Zero-Complaint Rule**: must provide full, refactored replacement code for any identified bottleneck or output `PASS: PERFORMANCE AUDIT CLEAN`.

---

## 5-Phase Execution Workflow

### Phase 1: Context Setup & Agent Initialization
- Codify system rules into `ai-rules.md`.
- Initialize Rust environment and verify compiler toolchain (`cargo`, `rustc`).
- Launch and verify active multi-agent triad.

### Phase 2: Code Architecture & Build Execution
- Code strictly from first principles using Rust's type system to eliminate error classes.
- Enforce strict $\le 250$ line limit per `.rs` file, split code at logical function boundaries, and use explicit domain-specific file naming.
- Embed comprehensive, structured `tracing` telemetry across all critical paths, state changes, warnings, and error boundaries.
- Maintain a clean developer experience with zero dead code.

### Phase 3: Deep Audit & Refactor Loop
- Execute an isolated "Critique-and-Fix" loop passing code to Security and Performance agents.
- Enforce **Zero-Complaint Rule**: agents must output full replacement code or explicit `PASS` state.

### Phase 4: Arbitration, Verification & Commit
- Strategic Arbiter resolves agent feedback, prioritizing Security over Performance.
- Run complete test ladder (`cargo test --workspace`, `cargo fmt`, `cargo clippy --workspace -- -D warnings`, `cargo audit`, `cargo udeps`).
- Bump semantic patch version, generate declarative commit messages, commit, and push to GitHub.

### Phase 5: Documentation & Deployment Pipeline
- Generate Blue Ocean `README.md` optimized for instant time-to-value (one-line install command + one perfect example).
- Deploy CI/CD via GitHub Actions. Build Alpine container images (`ash` retained) or native DEB/RPM/Unraid XML templates.
- Publish release assets and tags to GitHub Releases and GHCR.
