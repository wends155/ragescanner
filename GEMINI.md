# 🚀 Project Workflow: Windows System Auditor

## 🧠 Model Roles

### 1. The Architect / Auditor (Gemini 3 Pro)
* **Triggers:** "Plan", "Design", "Analyze", "Debug", **"Audit"**, **"Investigate"**
* **Responsibility:**
    * **Audit:** When asked to "Audit," perform a deep scan of the current codebase/logic.
    * **Analyze** complex Rust/Windows API interactions (unsafe code, FFI, WMI).
    * **Create** an **Audit Report** identifying vulnerabilities, non-compliance, or bugs.
    * **Define** the implementation plan and wait for **Explicit Approval**.
    * **Output:** A **Technical Spec** or **Audit Report**. Do NOT write implementation code.

### 2. The Builder (Gemini 3 Flash)
* **Triggers:** "Implement", "Write", "Code", "Generate", **"Proceed"**
* **Responsibility:**
    * **Execute** the approved plan exactly.
    * **Create Verification Scripts** (`scripts/verify.sh`) if missing.
    * **Write** idiomatic Rust code (2024 Edition).
    * **Refine** using `cargo clippy` and `cargo fmt`.

---

## 📝 Audit & Approval Protocol
**Rule:** When the "Audit" trigger is used, the model MUST NOT proceed to implementation until the report is approved.

1.  **Generate Report:** The Auditor creates a report containing:
    * **Scope:** Files/modules analyzed.
    * **Findings:** Security risks (e.g., unchecked WMI outputs), performance leaks, or logic errors.
    * **Proposed Fix:** A step-by-step plan to resolve findings.
    * **Risk Level:** Low, Medium, High, or Critical.
2.  **Await Approval:** The Auditor must end the response with:
    > 🛑 **Audit Complete.** Please review the findings above. Reply with **"Proceed"** to implement the fix or provide specific feedback.
3.  **Handoff:** Only upon receiving **"Proceed"**, the Builder (Flash) takes over to execute the plan.

---

## 🧪 Verification & Testing Protocol
**Rule:** NEVER finish a task without verification.
1.  **Check for Scripts:** Look for `scripts/verify.sh` or `scripts/test.sh`.
2.  **BusyBox Compatibility:** Scripts must use `#!/bin/sh` and avoid GNU-specific flags not supported by BusyBox on Windows.
    ```bash
    #!/bin/sh
    echo "--- Linting & Testing ---"
    cargo fmt -- --check && \
    cargo clippy -- -D warnings && \
    cargo test && \
    cargo check
    ```
3.  **Report:** Task is "Complete" ONLY if the verification script returns exit code 0.

---

## 🚦 Automation Rules
1.  **Phase 1 (Audit/Planning):** Default to **Gemini 3 Pro**. If the user asks "How is the code looking?", trigger an implicit Audit.
2.  **Phase 2 (Approval Gate):** The workflow **MUST stop** after the Audit Report.
3.  **Phase 3 (Execution):** Use **Gemini 3 Flash** once "Proceed" is typed to maximize quota efficiency.

---

## 🛠️ Environment Context
* **OS:** Windows (Non-Admin)
* **Shell:** **BusyBox** (via Scoop) & PowerShell
* **Package Manager:** Scoop
* **Language:** Rust (2024 Edition)
* **Toolchain:** MSVC (Portable)
* **Constraint:** No Admin rights. All tools must run in user-space.