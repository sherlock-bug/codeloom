pub mod git; pub mod queries; pub mod smart; pub mod tree_sitter;
pub mod clang;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_cpp() {
        assert_eq!(tree_sitter::detect_language("foo.cpp"), Some("cpp"));
        assert_eq!(tree_sitter::detect_language("foo.h"), Some("cpp"));
    }

    #[test]
    fn test_detect_language_python() {
        assert_eq!(tree_sitter::detect_language("foo.py"), Some("python"));
    }
}
