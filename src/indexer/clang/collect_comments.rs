//! Source-level comment extraction for C++ symbols and files.
//!
//! Clang's `-ast-dump=json` does not include comments in the output,
//! so we extract them directly from source files by scanning lines
//! near each symbol's location.

use std::fs;
use std::sync::Mutex;

/// Global line cache: file_path → Vec<line content>
static FILE_LINE_CACHE: std::sync::OnceLock<Mutex<std::collections::HashMap<String, Vec<String>>>> =
    std::sync::OnceLock::new();

fn get_file_lines(file_path: &str) -> Vec<String> {
    let cache = FILE_LINE_CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    let mut guard = cache.lock().unwrap();
    if let Some(lines) = guard.get(file_path) {
        return lines.clone();
    }
    if let Ok(content) = fs::read_to_string(file_path) {
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        guard.insert(file_path.to_string(), lines.clone());
        lines
    } else {
        Vec::new()
    }
}

/// Collect comments associated with a symbol at the given source location.
///
/// Returns concatenated comment text (prefixes stripped, lines joined with `\n`).
/// Collects three types of comments in order:
/// 1. **Above** — consecutive comment lines above the symbol (supports `///`, `/** */`, `//`)
/// 2. **Inline** — trailing comment on the declaration/definition line
/// 3. **Body** — comments inside the function body (between line_start and line_end)
///
/// `line_start` and `line_end` are 1-indexed. `line_end = 0` means unknown (no body).
pub fn collect_comments_for_symbol(
    file_path: &str,
    line_start: u32,
    line_end: u32,
) -> String {
    let lines = get_file_lines(file_path);
    if lines.is_empty() {
        return String::new();
    }

    let mut parts: Vec<String> = Vec::new();

    // 1. Above comments: scan upwards from line_start-2
    if line_start > 1 {
        collect_above_comments(&lines, line_start as usize, &mut parts);
    }

    // 2. Inline comment on the declaration line
    if line_start > 0 && line_start <= lines.len() as u32 {
        let decl_idx = line_start as usize - 1;
        if let Some(inline) = extract_inline_comment(&lines[decl_idx]) {
            if !parts.is_empty() {
                parts.push(String::new()); // blank line separator
            }
            parts.push(inline);
        }
    }

    // 3. Body comments (only for func/method with known body region)
    if line_end > line_start && line_end <= lines.len() as u32 {
        let body_start = line_start as usize;      // line after declaration
        let body_end = line_end as usize - 1;       // line before closing brace
        if body_start < body_end {
            let body_comments = collect_body_comments(&lines, body_start, body_end);
            if !body_comments.is_empty() {
                if !parts.is_empty() {
                    parts.push(String::new());
                }
                parts.push(body_comments);
            }
        }
    }

    parts.join("\n")
}

/// Collect file header comment — the first block of consecutive comment lines
/// at the top of the file, before any non-comment non-whitespace code.
pub fn collect_file_header(file_path: &str) -> String {
    let lines = get_file_lines(file_path);
    if lines.is_empty() {
        return String::new();
    }

    let mut comment_lines: Vec<String> = Vec::new();
    let mut in_comment = false;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Empty line inside header comment block keeps us in comment mode
            // but we stop collecting (header comments usually don't have blank lines)
            if in_comment {
                break; // blank line ends the file header
            }
            continue;
        }

        if trimmed.starts_with("///")
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("*")
        {
            in_comment = true;
            if let Some(text) = strip_comment_prefix(trimmed) {
                comment_lines.push(text);
            }
        } else if in_comment {
            // We were in a comment block, now hit code — stop
            break;
        } else {
            // First non-empty, non-comment line — not a comment file
            break;
        }
    }

    comment_lines.join("\n")
}

// ── Helpers ───────────────────────────────────────────────────────────

/// Scan upwards from `line_idx - 2` (skip the declaration line itself)
/// and collect consecutive comment lines.
fn collect_above_comments(lines: &[String], line_start: usize, parts: &mut Vec<String>) {
    let mut collected: Vec<String> = Vec::new();

    // Scan from line_idx - 2 (skip declaration line), going upwards
    let scan_start = line_start.saturating_sub(2); // 0-indexed
    let mut i = scan_start as i32;

    while i >= 0 {
        let trimmed = lines[i as usize].trim();
        if trimmed.is_empty() {
            // Skip blank lines between comment blocks
            i -= 1;
            continue;
        }
        if let Some(text) = strip_comment_prefix(trimmed) {
            collected.push(text);
            i -= 1;
        } else {
            break; // Non-comment line — stop
        }
    }

    // Reverse to restore original order (we scanned bottom-up)
    if !collected.is_empty() {
        collected.reverse();
        parts.extend(collected);
    }
}

/// Scan the function body region and collect all comment lines found inside.
fn collect_body_comments(lines: &[String], body_start: usize, body_end: usize) -> String {
    let mut comments: Vec<String> = Vec::new();

    for i in body_start..body_end {
        if i >= lines.len() {
            break;
        }
        let trimmed = lines[i].trim();
        if let Some(text) = strip_comment_prefix(trimmed) {
            comments.push(text);
        }
    }

    comments.join("\n")
}

/// Extract trailing inline comment from a line (after `//` or `/* */`).
fn extract_inline_comment(line: &str) -> Option<String> {
    // Look for // after code (not at the very start)
    let trimmed = line.trim();
    if let Some(pos) = trimmed.rfind("//") {
        // Skip if this looks like a URL
        let after = &trimmed[pos + 2..].trim();
        if !after.is_empty() && !after.starts_with("http") && !after.starts_with("www") {
            // Check that // is not inside a string literal
            let before = &trimmed[..pos];
            if !before.contains('"') && !before.contains('\'') {
                return Some(clean_inline(after));
            }
            // Even if there's a string before //, if // is after the string it's a valid comment
            return Some(clean_inline(after));
        }
    }
    None
}

fn clean_inline(text: &str) -> String {
    text.trim().to_string()
}

/// Strip comment prefix from a line. Returns `None` if the line is not a comment.
fn strip_comment_prefix(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if let Some(text) = trimmed.strip_prefix("///") {
        let t = text.trim();
        if t.is_empty() { return Some(String::new()); }
        return Some(t.to_string());
    }

    if let Some(text) = trimmed.strip_prefix("//!") {
        let t = text.trim();
        if t.is_empty() { return Some(String::new()); }
        return Some(t.to_string());
    }

    if let Some(text) = trimmed.strip_prefix("//") {
        let t = text.trim();
        if t.is_empty() { return Some(String::new()); }
        return Some(t.to_string());
    }

    // Multi-line comment: `/** ... */`, `/* ... */`, or `* ...`
    if trimmed.starts_with("/**") {
        // /** text */    or    /**
        //  * text          * text
        //  */              */
        let inner = trimmed
            .trim_start_matches("/**")
            .trim_end_matches("*/")
            .trim();
        if inner.is_empty() { return Some(String::new()); }
        return Some(inner.to_string());
    }

    if trimmed.starts_with("/*") {
        let inner = trimmed
            .trim_start_matches("/*")
            .trim_end_matches("*/")
            .trim();
        if inner.is_empty() { return Some(String::new()); }
        return Some(inner.to_string());
    }

    // Continuation of multi-line comment: `* text`
    if trimmed.starts_with('*') && !trimmed.starts_with("*/") {
        let text = trimmed.trim_start_matches('*').trim();
        if text.is_empty() { return Some(String::new()); }
        return Some(text.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_triple_slash() {
        assert_eq!(
            strip_comment_prefix("/// Opens a database.").unwrap(),
            "Opens a database."
        );
    }

    #[test]
    fn test_strip_double_slash() {
        assert_eq!(strip_comment_prefix("// inline").unwrap(), "inline");
    }

    #[test]
    fn test_strip_multiline_comment() {
        assert_eq!(
            strip_comment_prefix("/* Range check */").unwrap(),
            "Range check"
        );
    }

    #[test]
    fn test_strip_multiline_continuation() {
        assert_eq!(strip_comment_prefix(" * Performs compaction.").unwrap(), "Performs compaction.");
    }

    #[test]
    fn test_extract_inline_comment() {
        assert_eq!(
            extract_inline_comment("int x = 1;  // Maximum count").unwrap(),
            "Maximum count"
        );
    }

    #[test]
    fn test_not_comment() {
        assert!(strip_comment_prefix("int x = 1;").is_none());
        assert!(strip_comment_prefix("#include <cstdint>").is_none());
    }
}
