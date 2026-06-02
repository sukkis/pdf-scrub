use std::path::{Path, PathBuf};

pub fn write_output(
    content: &str,
    source_filename: &str,
    dest_dir: &Path,
) -> Result<PathBuf, String> {
    if content.contains("[NAME:") {
        return Err("anonymization incomplete".to_string());
    }

    let stem = Path::new(source_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("invalid filename")?;

    let output_path = dest_dir.join(format!("{stem}.md"));
    std::fs::write(&output_path, content).map_err(|_| "write failed".to_string())?;

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn name_tag_in_content_returns_error() {
        let dir = tempdir().unwrap();
        let result = write_output("[NAME: John Doe] some text", "doc.pdf", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn partial_name_tag_without_closing_bracket_returns_error() {
        let dir = tempdir().unwrap();
        let result = write_output(
            "text [NAME: John Doe without closing bracket",
            "doc.pdf",
            dir.path(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn clean_content_writes_file_and_returns_path() {
        let dir = tempdir().unwrap();
        let result = write_output("Clean content.", "doc.pdf", dir.path());
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "Clean content.");
    }

    #[test]
    fn output_filename_uses_pdf_stem() {
        let dir = tempdir().unwrap();
        let path = write_output("Content.", "contract.pdf", dir.path()).unwrap();
        assert_eq!(path.file_name().unwrap(), "contract.md");
    }

    #[test]
    fn existing_file_is_overwritten_silently() {
        let dir = tempdir().unwrap();
        write_output("First version.", "doc.pdf", dir.path()).unwrap();
        let path = write_output("Second version.", "doc.pdf", dir.path()).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "Second version.");
    }
}
