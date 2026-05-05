pub mod git; pub mod queries; pub mod smart; pub mod tree_sitter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_cpp() {
        assert_eq!(tree_sitter::detect_language("foo.cpp"), Some("cpp"));
        assert_eq!(tree_sitter::detect_language("foo.h"), Some("cpp"));
        assert_eq!(tree_sitter::detect_language("foo.cc"), Some("cpp"));
        assert_eq!(tree_sitter::detect_language("foo.hxx"), Some("cpp"));
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(tree_sitter::detect_language("foo.py"), Some("python"));
    }

    #[test]
    fn test_detect_language_java() {
        assert_eq!(tree_sitter::detect_language("foo.java"), Some("java"));
    }

    #[test]
    fn test_detect_language_typescript() {
        assert_eq!(tree_sitter::detect_language("foo.ts"), Some("typescript"));
        assert_eq!(tree_sitter::detect_language("foo.tsx"), Some("typescript"));
    }

    #[test]
    fn test_detect_language_go() {
        assert_eq!(tree_sitter::detect_language("foo.go"), Some("go"));
    }

    #[test]
    fn test_detect_language_unknown() {
        assert_eq!(tree_sitter::detect_language("foo.txt"), None);
        assert_eq!(tree_sitter::detect_language("foo.rs"), None);
    }

    #[test]
    fn test_create_parser_cpp() {
        let p = tree_sitter::create_parser("cpp");
        assert!(p.is_some());
    }

    #[test]
    fn test_index_result_default() {
        let r = smart::IndexResult::default();
        assert_eq!(r.files_scanned, 0);
        assert_eq!(r.symbols_new, 0);
        assert_eq!(r.head_commit, None);
    }
}
