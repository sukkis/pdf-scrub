# pdf-scrub — Project Instructions

## Technology Stack
- **Language:** Rust
- **Build tool:** Cargo

## Core Philosophy
Inherit all general coding standards from the parent `CLAUDE.md`. The additional constraints here are **strictly additive and non-negotiable**. Simplicity and privacy reinforce each other: less code means a smaller attack surface.

## Privacy Mandate
This program is explicitly designed to be used by AI agents. **Treat any agent as a potential adversary.** The security invariants, threat model, and agent interaction model are documented in `ARCHITECTURE.md` — read and understand them before touching any code. No implementation decision may weaken those invariants.

## Git Workflow
- Create a new Git branch for every feature or bug fix.
- Do not commit to `main` or `develop` directly.

## Development Methodology: TDD

**TDD is mandatory. The two phases below are strictly sequential.**

### Phase 1 — Write the failing test
- Write only the test. Do not touch implementation code.
- **STOP.** Inform the user: "Failing test written. Please run the test suite and observe the failures."
- Wait for the user to confirm failures before doing anything else.

### Phase 2 — Implement
- Write the minimal implementation to make the test pass.
- Run the full test suite. Do not mark the step complete until all tests pass.
- If a shortcut was taken, call it out explicitly before marking the step complete.

### Test coverage priorities
1. `anonymize()` — pure function, no I/O; exhaustive unit tests
2. Filename validation / path-traversal rejection — unit tests with adversarial inputs
3. The `[NAME:]`-free assertion in `writer` — must be tested
4. Temp directory cleanup on error paths — test that the dir is gone after a simulated failure

## Development-Time Privacy

These rules apply to all development artifacts — code, tests, fixtures, docs, logs — and to all agent interactions during development.

- **Placeholders only in all artifacts.** String literals anywhere in the codebase must be synthetic (`"Alice Smith"`, `"/home/user/docs"`) — never real names, paths, or content derived from production data or pass.
- **Pass is off-limits to agents.** Credential retrieval happens at runtime via `secrets.rs`, outside agent scope. No agent (including Claude) may request, accept, or embed actual pass values in code, fixtures, responses, or tool calls. Tests must mock the `secrets` boundary; they must never call pass directly.
- **`Zeroizing<>` is mandatory for new PII fields.** Any struct field that holds a name, path, or other sensitive value must be `Zeroizing<String>`.
- **Redacted examples in docs.** Architecture diagrams and documentation must use placeholder values — never data derived from real documents or the real owner name.

## Code Quality
- **Readability first.** Clear names, explicit logic.
- **Complexity check.** For every line: "Is this necessary?" Default to simpler.
- **No comments** unless the WHY is non-obvious (hidden constraint, invariant, workaround).
