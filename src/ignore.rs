// .codeloomignore support — similar to .gitignore but for CodeLoom indexing
use std::path::Path;

/// Load ignore patterns from .codeloomignore in the repo root.
/// Returns empty vec if file doesn't exist.
pub fn load_patterns(repo_root: &str) -> Vec<String> {
    let path = Path::new(repo_root).join(".codeloomignore");
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Check if a path should be ignored based on patterns.
/// `file_path` should be relative to repo root when possible, or absolute.
pub fn is_ignored(file_path: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    for pat in patterns {
        if matches_pattern(file_path, pat) {
            return true;
        }
    }
    false
}

/// Simple glob matching logic:
///   "dir/"      → matches any path inside that directory
///   "*.ext"     → matches files with that extension
///   "prefix*"   → matches paths starting with prefix
///   "literal"   → substring match (for simple cases)
fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.ends_with('/') {
        // Directory pattern: "test/" matches "/test/" or starting with "test/"
        let dir = pattern.trim_end_matches('/');
        path.contains(&format!("/{}/", dir))
            || path.starts_with(&format!("{}/", dir))
            || path.starts_with(&format!("./{}/", dir))
    } else if pattern.starts_with('*') {
        // Extension/suffix pattern: "*.test.cpp" matches .test.cpp files
        path.ends_with(&pattern[1..])
    } else if pattern.ends_with('*') {
        // Prefix pattern: "generated*" matches anything starting with "generated"
        path.contains(&pattern[..pattern.len() - 1])
    } else {
        // Literal substring match
        path.contains(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_pattern() {
        assert!(matches_pattern("test/test_foo.cpp", "test/"));
        assert!(matches_pattern("./test/foo.cpp", "test/"));
        assert!(matches_pattern("/home/repo/test/foo.cpp", "test/"));
        assert!(!matches_pattern("src/test.cpp", "test/"));
    }

    #[test]
    fn test_ext_pattern() {
        assert!(matches_pattern("foo.test.cpp", "*.test.cpp"));
        assert!(!matches_pattern("foo.cpp", "*.test.cpp"));
        assert!(matches_pattern("bar.md", "*.md"));
    }

    #[test]
    fn test_prefix_pattern() {
        assert!(matches_pattern("src/generated_foo.cpp", "generated*"));
        assert!(!matches_pattern("src/normal.cpp", "generated*"));
    }

    #[test]
    fn test_substring() {
        assert!(matches_pattern("path/to/examples/demo.cpp", "examples"));
        assert!(matches_pattern("include/third_party/foo.h", "third_party"));
    }

    #[test]
    fn test_load_empty() {
        let patterns = load_patterns("/nonexistent/path");
        assert!(patterns.is_empty());
    }
}
