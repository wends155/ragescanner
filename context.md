# 🗺️ Project Context: RageScanner

> **AI Instructions:** This file is the Source of Truth. Update this file during the **Phase 4: Summarize** stage of the TARS workflow.

---

## 🏗️ System Overview
* **Goal:** A high-performance, asynchronous IP scanner for Windows built with Rust, NWG, and Tokio.
* **Core Stack:** Rust 2024, `native-windows-gui`, `tokio`, `windows-rs`.
* **Architecture Pattern:** Asynchronous GUI Application using a Bridge pattern to decouple NWG (Event-driven UI) from Tokio (Asynchronous Logic).

---

## 💻 Environment & Constraints
* **Host OS:** Windows (Non-Admin)
* **Shell Environment:** BusyBox (via Scoop) / PowerShell
* **Toolchain:** MSVC (Portable), Cargo, Rustup via Scoop
* **Deployment:** Standalone User-space binary
* **Strict Rules:**
    1. No `sudo`/Admin commands.
    2. Scripts must be `#!/bin/sh` (BusyBox compatible).
    3. GUI must remain responsive during high-concurrency network scans.

---

## 📍 Current State (Recursive Summary)

### 🛠️ Recent Changes (Last 3 Cycles)
1.  **2026-02-18/Core:** Centralized port definitions in `types.rs` with `COMMON_PORTS` dictionary and `port_label` helper.
2.  **2026-02-18/UI:** Integrated descriptive port labels into TUI detail popup and NWG results list (e.g., "RPC/EPMAP").
3.  **2026-02-18/Docs:** Created `spec.md` as the Behavioral Source of Truth. Performed first detailed audits of test coverage and error quality.
4.  **2026-02-17/TUI:** Implemented the `rageping` TUI binary using `ratatui` and `crossterm`. Followed provided mockup and added (c) WSALIGAN attribution.

### 🧩 Active Components & APIs
* `src/lib.rs`: Library entry point; re-exports `tui`, `bridge`, `net`, `scanner`, and `types`.
* `src/tui/`: Full TUI implementation (`app`, `event`, `theme`, `ui`).
* `src/bin/tui.rs`: New binary `rageping` for terminal usage.
* `src/main.rs`: Original NWG GUI binary entry point.

### 🛠️ Maintenance & Scripts
* `Makefile`: Central entry point for `check`, `run`, `test`, `build`, and `verify`.
* `scripts/verify.sh`: Comprehensive quality gate (Lint + Test + Build).
* `BLUEPRINT_TEMPLATE.md`: Standardized format for Architect's **Think Phase** audits (includes "Files to be modified" scope).

---

### 💻 Shell & Tooling Quirks
* **PowerShell `&&` Limitation:** The default shell on this host (PowerShell) does not support `&&` as a statement separator.
    * **Solution:** Run commands sequentially in separate tool calls. Do not use `&&` in `run_command` tools.

---

## 📜 Decision Log (The "Why")
* **2026-02-11:** Chose **Native Windows GUI (NWG)** over browser-based frameworks for a zero-dependency feel in user-space.
* **2026-02-11:** Implemented a **Bridge pattern** to resolve the conflict between NWG's single-threaded loop and Tokio's multi-threaded runtime.
* **2026-02-11:** Formalized **Architect reports** with a mandatory template to ensure consistent audits, risk assessment, and scope definition (including affected files).
* **2026-02-17:** Standardized on **Rust 2024 Edition** across `Cargo.toml` and documentation.
* **2026-02-17/Refactor:** Moved core logic to a library to support multi-frontend development (TUI/CLI).
* **2026-02-17/Observability:** Implemented profile-based log levels and promoted ARP warnings to errors to ensure production visibility of network failures.
* **2026-02-18/Architecture:** Established `types.rs` as the single source of truth for shared metadata (e.g., `COMMON_PORTS`) to ensure consistency across multiple frontends.
* **2026-02-18/Governance:** Formalized `spec.md` as the Behavioral Source of Truth to comply with `GEMINI.md` requirements and provide a refactoring baseline.

---

## 🚧 Technical Debt & Pending Logic
* **Test Gaps**: 11 uncovered scenarios including high-priority offline/error paths in the scanner and TUI filter logic.
* **Error Context**: 72% of system errors (e.g. `scanner.rs:173`) are bare strings lacking IP/function context.
* **Next Steps**:
    *   Fix high-priority scanner test gaps (G7, G8).
    *   Implement recommended error message improvements (E1-E3).
    *   Implement user-configurable port lists.

---

## 🧪 Verification Commands
```bash
# Full Quality Gate
make verify

# Manual Lint Check (Sequential)
cargo fmt -- --check
cargo clippy -- -D warnings
```