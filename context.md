# 🗺️ Project Context: RageScanner

> **AI Instructions:** This file is the Source of Truth. Update this file during the **Phase 4: Summarize** stage of the TARS workflow.

---

## 🏗️ System Overview
* **Goal:** A high-performance, asynchronous IP scanner for Windows built with Rust, NWG, and Tokio.
* **Core Stack:** Rust 2021/2024, `native-windows-gui`, `tokio`, `windows-rs`.
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
*This section is updated by the Architect after every successful implementation.*

### 🛠️ Recent Changes (Last 3 Cycles)
1.  **2026-02-11/Baseline:** Initial project audit completed. `context.md` and `GEMINI.md` synchronized with technical stack, scripts, and absolute paths.

### 🧩 Active Components & APIs
* `src/main.rs`: Entry point, logging (`simplelog`), and panic hooks.
* `src/ui.rs`: GUI layout defined via `native-windows-gui`.
* `src/bridge.rs`: Orchestrates communication between the UI and background workers.
* `src/scanner.rs`: Implements the asynchronous IP scanning engine.
* `src/net.rs`: Low-level network primitives and DNS/MAC lookup utilities.
* `src/types.rs`: Common data structures.

### 🛠️ Maintenance & Scripts
* `Makefile`: Central entry point for `check`, `run`, `test`, `build`, and `verify`.
* `scripts/verify.sh`: Comprehensive quality gate (Lint + Test + Build).
* `BLUEPRINT_TEMPLATE.md`: Standardized format for Architect's **Think Phase** audits.

---

### 💻 Shell & Tooling Quirks
* **PowerShell `&&` Limitation:** The default shell on this host (PowerShell) does not support `&&` as a statement separator. Attempting to chain commands with `&&` results in a parser error.
    * **Solution:** Run commands sequentially in separate tool calls or use `;` (if appropriate for the task logic). In `run_command` tools, always prefer separate calls for multi-stage processes (e.g., `git add` followed by `git commit`).

---

## 📜 Decision Log (The "Why")
*Records why specific paths were taken to prevent circular reasoning in future "Think" phases.*

* **2026-02-11:** Chose **Native Windows GUI (NWG)** over browser-based frameworks to maintain a lightweight, zero-dependency feel.
* **2026-02-11:** Implemented a **Bridge pattern** to resolve the conflict between NWG's single-threaded loop and Tokio's multi-threaded runtime.

---

## 🚧 Technical Debt & Pending Logic
* **Known Issues:** None identified in initial audit.
* **Next Steps:** Implement multi-threaded scanning and real-time UI updates.

---

## 🧪 Verification Commands
*Standard commands the Executor must run to pass the Quality Gate.*

```bash
# Linting & Verification
make verify

# Manual Check
cargo fmt -- --check && cargo clippy -- -D warnings
```