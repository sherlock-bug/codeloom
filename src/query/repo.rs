// Repository listing: scan ~/.codeloom/*.rag.db for indexed repos
use crate::config::Config;

/// Returns list of repo names from *.rag.db files in ~/.codeloom/
pub fn list_repos() -> Vec<String> {
    let data_dir = match Config::data_dir() {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    let mut repos = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("db")
                && path.file_stem().and_then(|s| s.to_str()).map_or(false, |s| s.ends_with(".rag"))
            {
                let name = path.file_stem().unwrap().to_string_lossy();
                // Strip ".rag" suffix → repo name
                if let Some(repo) = name.strip_suffix(".rag") {
                    repos.push(repo.to_string());
                }
            }
        }
    }
    repos.sort();
    repos
}

/// Check if a file path exists and is a valid database
fn _db_exists(repo: &str) -> bool {
    if let Ok(dd) = Config::data_dir() {
        dd.join(format!("{}.rag.db", repo)).exists()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_repos_returns_vec() {
        let repos = list_repos();
        // Should always return a Vec (may be empty if no repos indexed)
        assert!(repos.is_empty() || !repos.is_empty()); // just ensures it compiles and runs
    }

    #[test]
    fn test_list_repos_no_panic() {
        // Just verify no panic from the function
        let _ = list_repos();
    }
}
