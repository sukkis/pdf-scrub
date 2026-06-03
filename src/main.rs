use clap::Parser;
use pdf_scrub::{anonymize, ollama, pages, secrets, writer};
use std::path::Path;

#[derive(Parser)]
struct Args {
    filename: String,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args = Args::parse();

    let secrets = secrets::load()?;

    let source_path = pages::resolve_source_path(&args.filename, Path::new(&*secrets.source_dir))?;

    let tempdir = tempfile::tempdir().map_err(|_| "error: internal error".to_string())?;

    let page_paths = pages::run_pdftoppm(&source_path, tempdir.path())?;

    let mut page_texts = Vec::new();
    for page_path in &page_paths {
        let image_bytes =
            std::fs::read(page_path).map_err(|_| "error: internal error".to_string())?;
        let text = ollama::ask_ollama(&image_bytes, &secrets.model).await?;
        page_texts.push(text);
    }

    let joined = page_texts.join("\n\n---\n\n");
    let clean = anonymize::anonymize(&joined, &secrets.owner_firstname, &secrets.owner_lastname);

    let output_path = writer::write_output(&clean, &args.filename, Path::new(&*secrets.dest_dir))?;

    println!("{}", output_path.display());
    Ok(())
}
