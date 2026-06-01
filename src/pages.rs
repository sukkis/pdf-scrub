use std::path::{Path, PathBuf};

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
