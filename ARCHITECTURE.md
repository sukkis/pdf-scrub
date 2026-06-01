# Architecture: pdf-scrub

## Overview

```
agent
  │
  │  bare filename
  ▼
main.rs ──── secrets.rs ──── pass
  │              (source-dir, dest-dir,
  │               owner-name, model)
  │
  ├─ pages.rs
  │    └─ pdftoppm → PNG files in 0700 tempdir
  │
  ├─ ollama.rs  (one call per page)
  │    └─ raw markdown with [NAME: ...] tags
  │
  ├─ anonymize.rs  (pure function, document-scoped)
  │    └─ anonymized markdown, no [NAME:] tags
  │
  └─ writer.rs
       └─ assert no [NAME:] → write dest-dir/<stem>.md
  │
  │  output path
  ▼
agent
```

## Project Structure

Single crate with `lib.rs` and `main.rs`. No workspace.

```
pdf-scrub/
├── Cargo.toml
└── src/
    ├── main.rs        # CLI entrypoint: clap wiring, top-level orchestration
    ├── lib.rs         # Re-exports modules; public API surface for tests
    ├── secrets.rs     # All pass reads; single module owns the pass interface
    ├── pages.rs       # Shells out to pdftoppm; manages 0700 tempdir lifecycle
    ├── ollama.rs      # HTTP calls to Ollama; one call per page image
    ├── anonymize.rs   # Pure anonymization function; no I/O
    └── writer.rs      # Final safety assertion and output file write
```

## Module Responsibilities

### `secrets`

Reads all configuration from pass via `getfrompass`. Single point of contact with the secret store. Returns a `Secrets` struct populated at startup; nothing else in the program calls `getfrompass` directly.

```rust
pub struct Secrets {
    pub source_dir: Zeroizing<String>,
    pub dest_dir:   Zeroizing<String>,
    pub owner_name: Zeroizing<String>,
    pub model:      String,            // not sensitive; fallback to "mistral-small3.2:24b"
}
```

Fails with `error: configuration incomplete` if `source-dir`, `dest-dir`, or `owner-name` are absent from pass.

### `pages`

Validates the agent-supplied filename (see Security Invariants), resolves the full source path, then shells out to `pdftoppm` via `std::process::Command`:

```
pdftoppm -r 200 -png <source_path> <tempdir>/page
→ <tempdir>/page-1.png, page-2.png, …
```

Creates the tempdir with mode `0700`. Returns a sorted list of PNG paths (numerical sort on the page number extracted from the filename). Drops a guard that unconditionally deletes the tempdir — the guard is created before `pdftoppm` is invoked so cleanup runs even if the subprocess fails.

### `ollama`

Sends one multimodal request to `http://localhost:11434/api/generate` per page image. The prompt is hardcoded. The image is base64-encoded and passed in the `images` array.

```json
{
  "model": "<model>",
  "prompt": "Extract the text from this document page as Markdown. Mark every person name you find with the tag [NAME: Firstname Lastname]. Do not substitute or remove anything else. Preserve all other content exactly.",
  "images": ["<base64 PNG>"],
  "stream": false
}
```

Returns the raw `response` string from the JSON reply. Fails with `error: ollama unavailable` on any network or HTTP error.

### `anonymize`

Pure function; no I/O. Called once with the concatenated raw markdown for the whole document and the owner name. Accumulates the name→replacement mapping in a `Vec` for stable numbering across pages.

```rust
pub fn anonymize(text: &str, owner_name: &str) -> String
```

Replacement rules applied in order per `[NAME: ...]` tag:

1. Tagged name matches `owner_name` exactly (case-insensitive) → `Omistaja`
2. Tagged name contains the owner's first name token or last name token as a whole word (case-insensitive) → `Omistaja?`
3. Any other name → `henkilö 1`, `henkilö 2`, … (assigned on first occurrence, consistent for the lifetime of the `Vec`)

### `writer`

Receives the fully anonymized markdown string. Before writing:

- Asserts that the string contains no `[NAME:` substring. Fails hard if found — this indicates a bug in `anonymize` and must never produce output.

Writes the result to `dest_dir/<stem>.md`, overwriting any existing file. Returns the absolute output path.

## Data Flow

```
filename (bare)
  → validate (no path separators, no traversal)
  → resolve: source_dir / filename
  → pdftoppm → [page-1.png, page-2.png, …] in 0700 tempdir
  → for each page (numerical order):
      ollama() → raw_page_md
  → join pages with "\n\n---\n\n"
  → anonymize(joined, owner_name) → clean_md
  → assert no [NAME: in clean_md
  → write dest_dir/<stem>.md
  → delete tempdir
  → print output path
```

## Security Invariants

1. **Source path never disclosed.** Never printed, logged, or included in any error message or output.
2. **No raw text to output.** Anonymization is a mandatory gate. The `writer` assertion is the last line of defence.
3. **No path traversal.** Filename is rejected if it contains `/`, `\`, or `..`, or if the resolved canonical path does not start with the canonical source directory.
4. **Temp files in 0700 dir, always destroyed.** The drop guard runs on all paths including panics.
5. **Ollama endpoint is hardcoded.** `http://localhost:11434` — not configurable via CLI or environment.
6. **No outbound network except Ollama.** No other connections are made.
7. **Fail hard on malformed tags.** Any `[NAME:` without a closing `]` in Ollama output is an error; the document is not written.
8. **Minimal error messages.** Errors visible to the agent carry no variable content derived from the document or filesystem.

## Secret Management

All access via `getfrompass`. All entries are under the `machine/pdf-scrub/` namespace — the program reads them at runtime.

| Pass entry | Content | Required |
|---|---|---|
| `machine/pdf-scrub/source-dir` | Absolute path to source PDF directory | Yes |
| `machine/pdf-scrub/dest-dir` | Absolute path to output markdown directory | Yes |
| `machine/pdf-scrub/owner-name` | Full owner name as it appears in documents | Yes |
| `machine/pdf-scrub/model` | Ollama model name | No — defaults to `mistral-small3.2:24b` |

Source and destination directories must never be the same path.

## Dependencies

| Crate | Purpose | Justification |
|---|---|---|
| `clap` | CLI argument parsing | Standard choice |
| `getfrompass` | All pass reads | Sole interface to secrets; already used across this workspace |
| `reqwest` + `tokio` | HTTP to local Ollama | `std` has no HTTP/TLS |
| `serde_json` | Ollama request/response JSON | `std` has no JSON |
| `base64` | Encode PNG images for Ollama multimodal API | Single-purpose, small |
| `tempfile` | 0700 tempdir with RAII cleanup | Correct-by-construction temp file hygiene |

System dependency: `pdftoppm` from `poppler-utils`.

No other dependencies without explicit discussion and justification.

## TDD Strategy

### Module implementation order

1. `anonymize` — pure function, no I/O; exhaustive unit tests first
2. `pages` — filename validation and path-traversal rejection; unit tests with adversarial inputs
3. `writer` — the `[NAME:]`-free assertion; test that it fails when tags remain
4. `ollama` — HTTP calls; integration test against a running Ollama instance
5. `secrets` — pass reads; test with known pass entries
6. `main` — end-to-end wiring

## Design Decisions

| Question | Decision |
|---|---|
| Output delivery | Write to `dest-dir`, return path — not stdout |
| Output filename | Derived from input stem (`contract.pdf` → `contract.md`) |
| Overwrite behaviour | Silent overwrite |
| Name mapping scope | Document-scoped `Vec`; reset per run |
| Owner partial match | First/last name token (whole word); initials deferred to v2 |
| Page separator | `---` only; no page numbers |
| Ollama calls | One call per page (OCR + tagging combined) |
| Ollama prompt | Hardcoded |
| Ollama model | From pass with fallback to `mistral-small3.2:24b` |
| Ollama endpoint | Hardcoded to `http://localhost:11434` |
| Page image resolution | Hardcoded at 200 DPI |
| Malformed Ollama output | Fail hard — no partial anonymization |
| CLI path override | None — destination solely from pass |
