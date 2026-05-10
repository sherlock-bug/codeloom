//! compile_commands.json discovery and parsing.
//!
//! Search order: explicit path → project root → build/ → out/
//! Falls back to single-file mode if not found.

use std::collections::HashMap;
use std::path::Path;

/// Map: file path → clang arguments (without compiler, -c, -o flags)
pub type CompileCommands = HashMap<String, Vec<String>>;

/// Discover compile_commands.json using the search order.
/// Returns empty map if not found (single-file fallback mode).
pub fn discover(explicit: Option<&str>, project_root: &str) -> anyhow::Result<CompileCommands> {
    // Check env var as implicit fallback
    let env_cc = std::env::var("CODELOOM_COMPILE_COMMANDS").ok();
    let explicit = explicit.or(env_cc.as_deref());
    if let Some(p) = explicit {
        return parse(Path::new(p));
    }

    // 2. Project root → build/ → out/
    let root = Path::new(project_root);
    for candidate in &["compile_commands.json", "build/compile_commands.json", "out/compile_commands.json"] {
        let path = root.join(candidate);
        if path.exists() {
            return parse(&path);
        }
    }

    // Not found: return empty → single-file fallback
    Ok(CompileCommands::new())
}

/// Parse a compile_commands.json file.
/// Extracts directory, command/arguments, and filters compiler-specific flags.
fn parse(path: &Path) -> anyhow::Result<CompileCommands> {
    let content = std::fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    let arr = json.as_array()
        .ok_or_else(|| anyhow::anyhow!("compile_commands.json is not an array"))?;
    
    let mut map = CompileCommands::new();
    
    for entry in arr {
        let file = entry["file"].as_str().unwrap_or("");
        if file.is_empty() {
            continue;
        }
        
        let dir = entry["directory"].as_str().unwrap_or(".");
        let full_path = if Path::new(file).is_absolute() {
            file.to_string()
        } else {
            Path::new(dir).join(file).to_string_lossy().to_string()
        };
        
        // Extract args from "command" or "arguments"
        let args: Vec<String> = if let Some(cmd_str) = entry["command"].as_str() {
            shlex_like_split(cmd_str)
        } else if let Some(args_arr) = entry["arguments"].as_array() {
            args_arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        } else {
            continue;
        };
        
        // Filter: keep -I, -D, -std=, skip compiler, -c, -o
        let filtered: Vec<String> = args.iter()
            .filter(|a| {
                let s = a.as_str();
                s.starts_with("-I") || s.starts_with("-D") || s.starts_with("-std=") ||
                s.starts_with("-m") || s.starts_with("-f") || s.starts_with("-W") ||
                s.starts_with("-isystem") || s.starts_with("-include")
            })
            .cloned()
            .collect();
        
        map.insert(full_path, filtered);
    }
    
    Ok(map)
}

/// Simple shell-like split (handles quotes minimally).
fn shlex_like_split(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    
    for ch in cmd.chars() {
        match ch {
            '"' => in_quote = !in_quote,
            ' ' if !in_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shlex_split() {
        let args = shlex_like_split("clang++ -I/include -std=c++17 -c src/main.cpp");
        assert!(args.contains(&"-I/include".to_string()));
        assert!(args.contains(&"-std=c++17".to_string()));
    }

    #[test]
    fn test_filter_keeps_includes() {
        let args: Vec<String> = ["clang++", "-I/include", "-std=c++17", "-c", "file.cpp"]
            .iter().map(|s| s.to_string()).collect();
        let filtered: Vec<_> = args.iter()
            .filter(|a| a.starts_with("-I") || a.starts_with("-std="))
            .collect();
        assert_eq!(filtered.len(), 2);
    }
}
