# pdf-scrub

Converts a PDF to Markdown, anonymising all person names in the process. Runs entirely locally — no data leaves the machine.

## Requirements

- [Rust](https://rustup.rs/) (to build)
- [Pass](https://www.passwordstore.org/) with GnuPG initialised
- [poppler-utils](https://poppler.freedesktop.org/) (`pdftoppm`)
- [Ollama](https://ollama.com/) running locally with a vision-capable model

```
sudo apt-get install -y poppler-utils pass gnupg
ollama pull mistral-small3.2:24b
```

## Installation

```
cargo install --path .
```

## Setup

pdf-scrub reads all configuration from `pass`. Set up the following four entries before first use:

### Required

```
pass insert machine/pdf-scrub/source-dir
```
The absolute path to the directory where your source PDF files are stored. This directory should be mode `0700` — only you and the program should be able to read it.

```
pass insert machine/pdf-scrub/dest-dir
```
The absolute path to the directory where anonymised Markdown files will be written. Must be different from `source-dir`.

```
pass insert machine/pdf-scrub/owner-name
```
Your full name exactly as it appears in your documents (e.g. `Matti Meikäläinen`). This name will be replaced with `Omistaja` in the output. First or last name alone will be replaced with `Omistaja?`.

### Optional

```
pass insert machine/pdf-scrub/model
```
The Ollama model to use for OCR. Defaults to `mistral-small3.2:24b` if not set.

## Usage

```
pdf-scrub <filename>
```

`<filename>` is the bare name of the PDF (e.g. `contract.pdf`), not a full path. The program looks for the file in `machine/pdf-scrub/source-dir`.

On success, prints the path of the written Markdown file.

## Anonymisation

All person names found in the document are replaced:

| Detected name | Replacement |
|---|---|
| Your full name (exact match) | `Omistaja` |
| Your first or last name alone | `Omistaja?` |
| Any other person | `henkilö 1`, `henkilö 2`, … |

Numbering of other persons is stable within a document — the same person always gets the same number throughout.
