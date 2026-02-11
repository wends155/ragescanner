# 🚀 Project Workflow: Windows System Auditor

## 🧠 Model Roles

### 1. The Architect (Gemini 3 Pro)
* **Triggers:** "Plan", "Design", "Analyze", "Debug", **"Investigate"**
* **Responsibility:**
    * **Analyze** complex Rust/Windows API interactions (unsafe code, FFI, WMI).
    * **Investigate** unknown errors or architecture bottlenecks.
    * **Create** detailed, step-by-step implementation plans.
    * **Define** the verification strategy (e.g., "Create a script to mock the Registry").
    * **Output:** A **Technical Spec**, **Checklist**, or **Debug Strategy**. Do NOT write full implementation code.

### 2. The Builder (Gemini 3 Flash)
* **Triggers:** "Implement", "Write", "Code", "Generate", **"Proceed"**
* **Responsibility:**
    * **Execute** the Architect's plan exactly.
    * **Create Verification Scripts** (`scripts/verify.sh`) if they do not exist.
    * **Write** the actual Rust code.
    * **Refine** code using `cargo fmt` and `cargo clippy` standards.

---

## 🧪 Verification & Testing Protocol
**Rule:** NEVER finish a task without verification.
1.  **Check for Scripts:** Look for a `scripts/verify.sh` or `scripts/test.sh`.
2.  **Create if Missing:** If no script exists, create a portable shell script (compatible with **BusyBox**) containing:
    ```bash
    #!/bin/sh
    echo "Running Format Check..."
    cargo fmt -- --check
    echo "Running Linter..."
    cargo clippy -- -D warnings
    echo "Running Tests..."
    cargo test
    echo "Running Build Check..."
    cargo check
    ```
3.  **Execute:** Run the script after every major code change.
4.  **Report:** Only mark the task as "Complete" if `cargo check`, `clippy`, and `test` pass.

---

## 🚦 Automation Rules
1.  **Phase 1 (Planning):** If the request implies deep reasoning (e.g., "Investigate why WMI is slow"), automatically use **Gemini 3 Pro**.
2.  **Phase 2 (Hand-off):** When I say **"Proceed"**, switch to **Gemini 3 Flash** to implement the plan and run the verification script.
3.  **Quota Saver:** For simple fixes (typos, comments, one-line changes), default to **Flash**.

---

## 🛠️ Environment Context
* **OS:** Windows (Non-Admin)
* **Shell:** **BusyBox** (sh/bash compatible) & PowerShell
* **Package Manager:** Scoop
* **Language:** Rust (2024 Edition)
* **Toolchain:** MSVC (Portable via Scoop)
