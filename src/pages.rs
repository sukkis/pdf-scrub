use std::path::{Path, PathBuf};

pub fn resolve_source_path(filename: &str, source_dir: &Path) -> Result<PathBuf, String> {
    if !validate_filename(filename) {
        return Err("invalid document name".to_string());
    }
    let candidate = source_dir.join(filename);
    let canonical_dir = source_dir
        .canonicalize()
        .map_err(|_| "invalid document name".to_string())?;
    let canonical = candidate
        .canonicalize()
        .map_err(|_| "invalid document name".to_string())?;
    if !canonical.starts_with(&canonical_dir) {
        return Err("invalid document name".to_string());
    }
    Ok(canonical)
}

pub fn run_pdftoppm(source_path: &Path, tempdir: &Path) -> Result<Vec<PathBuf>, String> {
    let status = std::process::Command::new("pdftoppm")
        .args(["-r", "200", "-png"])
        .arg(source_path)
        .arg(tempdir.join("page"))
        .status()
        .map_err(|_| "pdftoppm not available".to_string())?;
    if !status.success() {
        return Err("pdftoppm failed".to_string());
    }
    let pages: Vec<PathBuf> = std::fs::read_dir(tempdir)
        .map_err(|_| "pdftoppm failed".to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|ext| ext == "png").unwrap_or(false))
        .collect();
    if pages.is_empty() {
        return Err("pdftoppm produced no pages".to_string());
    }
    Ok(sort_page_paths(pages))
}

pub fn validate_filename(name: &str) -> bool {
    if name.is_empty() || name == "." || name == ".." {
        return false;
    }
    !name.contains('/') && !name.contains('\\') && !name.contains('\0')
}

pub fn sort_page_paths(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort_by_key(|p| page_number(p).unwrap_or(0));
    paths
}

fn page_number(path: &Path) -> Option<u32> {
    let stem = path.file_stem()?.to_str()?;
    stem.rsplit('-').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_filename ---

    #[test]
    fn plain_filename_is_valid() {
        assert!(validate_filename("contract.pdf"));
    }

    #[test]
    fn filename_with_spaces_is_valid() {
        assert!(validate_filename("my document.pdf"));
    }

    #[test]
    fn filename_with_unicode_is_valid() {
        assert!(validate_filename("café.pdf"));
    }

    #[test]
    fn empty_filename_is_rejected() {
        assert!(!validate_filename(""));
    }

    #[test]
    fn forward_slash_is_rejected() {
        assert!(!validate_filename("subdir/file.pdf"));
    }

    #[test]
    fn backslash_is_rejected() {
        assert!(!validate_filename("subdir\\file.pdf"));
    }

    #[test]
    fn absolute_path_is_rejected() {
        assert!(!validate_filename("/etc/passwd"));
    }

    #[test]
    fn double_dot_traversal_is_rejected() {
        assert!(!validate_filename("../secret.pdf"));
    }

    #[test]
    fn double_dot_alone_is_rejected() {
        assert!(!validate_filename(".."));
    }

    #[test]
    fn single_dot_is_rejected() {
        assert!(!validate_filename("."));
    }

    #[test]
    fn null_byte_is_rejected() {
        assert!(!validate_filename("file\0.pdf"));
    }

    // --- resolve_source_path ---

    #[test]
    fn valid_filename_resolves_to_source_dir() {
        let dir = tempfile::tempdir().unwrap();
        let pdf = dir.path().join("contract.pdf");
        std::fs::write(&pdf, b"").unwrap();
        let result = resolve_source_path("contract.pdf", dir.path());
        assert_eq!(result.unwrap(), pdf);
    }

    #[test]
    fn invalid_filename_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve_source_path("../escape.pdf", dir.path()).is_err());
    }

    #[test]
    fn nonexistent_file_returns_err() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve_source_path("missing.pdf", dir.path()).is_err());
    }

    // --- sort_page_paths ---

    #[test]
    fn pages_are_sorted_numerically_not_lexicographically() {
        let paths = vec![
            PathBuf::from("/tmp/page-10.png"),
            PathBuf::from("/tmp/page-2.png"),
            PathBuf::from("/tmp/page-1.png"),
        ];
        let sorted = sort_page_paths(paths);
        assert_eq!(
            sorted,
            vec![
                PathBuf::from("/tmp/page-1.png"),
                PathBuf::from("/tmp/page-2.png"),
                PathBuf::from("/tmp/page-10.png"),
            ]
        );
    }

    #[test]
    fn single_page_is_returned_unchanged() {
        let paths = vec![PathBuf::from("/tmp/page-1.png")];
        let sorted = sort_page_paths(paths);
        assert_eq!(sorted, vec![PathBuf::from("/tmp/page-1.png")]);
    }
}
