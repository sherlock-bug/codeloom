// Document indexing: parse markdown into doc_nodes for embedding-based code-doc linking
use rusqlite::Connection;
pub mod glossary;

/// Store a markdown file's content as doc_nodes (split by ##/### sections)
pub fn index_markdown(conn: &Connection, path: &str, content: &str, repo: &str) -> anyhow::Result<usize> {
    let mut count = 0;
    let mut current_title = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let mut current_section = String::new();
    let mut current_level = 0;
    let mut current_content = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            if !current_content.trim().is_empty() || !current_section.is_empty() {
                store_section(conn, path, repo, &current_title, &current_section, current_level, &current_content)?;
                count += 1;
            }
            current_section = trimmed[3..].to_string();
            current_level = 2;
            current_content = String::new();
        } else if trimmed.starts_with("### ") {
            if !current_content.trim().is_empty() || !current_section.is_empty() {
                store_section(conn, path, repo, &current_title, &current_section, current_level, &current_content)?;
                count += 1;
            }
            current_section = trimmed[4..].to_string();
            current_level = 3;
            current_content = String::new();
        } else if trimmed.starts_with("# ") {
            current_title = trimmed[2..].to_string();
            current_section = String::new();
            current_content = String::new();
        } else {
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }
    if !current_content.trim().is_empty() || !current_section.is_empty() {
        store_section(conn, path, repo, &current_title, &current_section, current_level, &current_content)?;
        count += 1;
    }
    // Also store the full document
    let hash = crate::storage::dedup::hash_content(&format!("{}:{}:{}", current_title, "", content));
    conn.execute(
        "INSERT INTO doc_nodes (repo, title, section_path, content, level, file_path, file_format, content_hash)
         VALUES (?1, ?2, '', ?3, 1, ?4, 'md', ?5)
         ON CONFLICT(repo, file_path, section_path) DO UPDATE SET
         title=excluded.title, content=excluded.content, level=excluded.level,
         file_format=excluded.file_format, content_hash=excluded.content_hash",
        rusqlite::params![repo, current_title, content, path, hash],
    )?;
    count += 1;
    Ok(count)
}

fn store_section(conn: &Connection, path: &str, repo: &str, title: &str, section: &str, level: i32, content: &str) -> anyhow::Result<()> {
    let hash = crate::storage::dedup::hash_content(&format!("{}:{}:{}", title, section, content));
    conn.execute(
        "INSERT INTO doc_nodes (repo, title, section_path, content, level, file_path, file_format, content_hash)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'md', ?7)
         ON CONFLICT(repo, file_path, section_path) DO UPDATE SET
         title=excluded.title, content=excluded.content, level=excluded.level,
         file_format=excluded.file_format, content_hash=excluded.content_hash",
        rusqlite::params![repo, title, section, content, level, path, hash],
    )?;
    Ok(())
}
