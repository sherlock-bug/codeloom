// FTS5 full-text index — BM25 keyword search on symbols and documents
use rusqlite::Connection;

/// Mobile-friendly search result (no String cloning in hot paths)
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub rowid: i64,
    pub score: f64,
    pub hit_type: HitType,
    pub name: String,
    pub file_path: String,
    pub line_start: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HitType {
    Code,
    Doc,
}

/// Fill FTS5 symbol index from symbols table (filtered by repo)
pub fn fill_symbols_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> {
    // Clear old data first (FTS5 doesn't support WHERE DELETE on content tables)
    conn.execute("DELETE FROM fts5_sym", [])?;

    let count = conn.execute(
        "INSERT INTO fts5_sym(name, file_path, signature)
         SELECT name, file_path, COALESCE(signature, '') FROM symbols WHERE repo=?1 ORDER BY rowid",
        rusqlite::params![repo],
    )?;
    Ok(count)
}

/// Fill FTS5 document index from doc_nodes table (filtered by repo)
pub fn fill_docs_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> {
    conn.execute("DELETE FROM fts5_doc", [])?;

    let count = conn.execute(
        "INSERT INTO fts5_doc(title, section_path, content)
         SELECT COALESCE(title, ''), COALESCE(section_path, ''), content FROM doc_nodes WHERE repo=?1 ORDER BY rowid",
        rusqlite::params![repo],
    )?;
    Ok(count)
}

/// Clear all FTS5 data
pub fn clear_fts(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("DELETE FROM fts5_sym; DELETE FROM fts5_doc;")?;
    Ok(())
}

/// FTS5 BM25 keyword search on symbols only (used by hybrid search)
pub fn search_symbols(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    // Escape FTS5 special characters to avoid syntax errors
    let safe_query = escape_fts5(query);

    let sql = "SELECT fts5_sym.rowid, bm25(fts5_sym) as score, s.name, s.file_path, s.line_start
               FROM fts5_sym
               JOIN symbols s ON s.rowid = fts5_sym.rowid
               JOIN branches b ON b.symbol_id = s.id
               WHERE fts5_sym MATCH ?1 AND s.repo=?2 AND b.branch_name=?3
               ORDER BY score
               LIMIT ?4";

    let mut stmt = conn.prepare(sql)?;
    let hits = stmt
        .query_map(
            rusqlite::params![safe_query, repo, branch, limit as i64],
            |r| {
                Ok(SearchHit {
                    rowid: r.get(0)?,
                    score: r.get(1)?,
                    hit_type: HitType::Code,
                    name: r.get(2)?,
                    file_path: r.get(3)?,
                    line_start: r.get(4)?,
                })
            },
        )?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hits)
}

/// FTS5 BM25 search on documents only
pub fn search_docs(
    conn: &Connection,
    query: &str,
    repo: &str,
    limit: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    let safe_query = escape_fts5(query);

    let sql = "SELECT fts5_doc.rowid, bm25(fts5_doc) as score, d.title, d.file_path, 0
               FROM fts5_doc
               JOIN doc_nodes d ON d.rowid = fts5_doc.rowid
               WHERE fts5_doc MATCH ?1 AND d.repo=?2
               ORDER BY score
               LIMIT ?3";

    let mut stmt = conn.prepare(sql)?;
    let hits = stmt
        .query_map(
            rusqlite::params![safe_query, repo, limit as i64],
            |r| {
                Ok(SearchHit {
                    rowid: r.get(0)?,
                    score: r.get(1)?,
                    hit_type: HitType::Doc,
                    name: r.get(2)?,
                    file_path: r.get(3)?,
                    line_start: r.get(4)?,
                })
            },
        )?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hits)
}

/// Escape FTS5 special characters to prevent query syntax errors
fn escape_fts5(query: &str) -> String {
    // FTS5 special chars: ^ * " - ( ) : AND OR NOT NEAR
    // Strategy: wrap each term in double quotes for literal matching,
    // but also allow multi-word queries by inserting AND between terms
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.len() == 1 {
        // Single term: use NEAR(0) trick for substring matching, or just quote it
        // For substring behavior similar to LIKE %term%, use prefix + quote
        let term = terms[0];
        // Quote the term for literal matching, and prepend * for prefix-like matching
        if term.chars().all(|c| c.is_alphanumeric() || c == '_') {
            format!("\"{}\" OR {}*", term, term)
        } else {
            format!("\"{}\"", term)
        }
    } else {
        // Multiple terms: AND them
        let quoted: Vec<String> = terms
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect();
        quoted.join(" AND ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        storage::migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn test_fill_and_search_symbols() {
        let conn = mem_db();

        // Insert test symbols
        conn.execute(
            "INSERT INTO symbols (repo, name, kind, definition, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'AuthService', 'class', 'class AuthService {}', 'h1', 'src/auth.cpp', 10, 15, 'class AuthService')",
            [],
        ).unwrap();
        let sym_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id],
        ).unwrap();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, definition, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'LoginManager', 'class', 'class LoginManager {}', 'h2', 'src/auth.cpp', 20, 25, 'class LoginManager')",
            [],
        ).unwrap();
        let sym_id2 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id2],
        ).unwrap();

        // Fill FTS5
        let count = fill_symbols_fts(&conn, "test").unwrap();
        assert_eq!(count, 2);

        // Search exact
        let hits = search_symbols(&conn, "AuthService", "test", "main", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "AuthService");

        // Search partial
        let hits = search_symbols(&conn, "Auth", "test", "main", 10).unwrap();
        assert!(hits.iter().any(|h| h.name == "AuthService"));
    }

    #[test]
    fn test_fill_and_search_docs() {
        let conn = mem_db();

        conn.execute(
            "INSERT INTO doc_nodes (repo, title, section_path, content, level, file_path, file_format, branch_name)
             VALUES ('test', 'Getting Started', 'intro', 'This guide covers setup and configuration.', 1, 'docs/readme.md', 'md', 'main')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO doc_nodes (repo, title, section_path, content, level, file_path, file_format, branch_name)
             VALUES ('test', 'API Reference', 'api/auth', 'Authentication endpoints for login and logout.', 2, 'docs/api.md', 'md', 'main')",
            [],
        ).unwrap();

        let count = fill_docs_fts(&conn, "test").unwrap();
        assert_eq!(count, 2);

        let hits = search_docs(&conn, "setup", "test", 5).unwrap();
        assert!(hits.iter().any(|h| h.name.contains("Getting Started")));

        let hits = search_docs(&conn, "authentication", "test", 5).unwrap();
        assert!(hits.iter().any(|h| h.name.contains("API Reference")));
    }

    #[test]
    fn test_multi_word_search() {
        let conn = mem_db();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, definition, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'memory_alloc', 'function', 'void* memory_alloc(size_t n)', 'h3', 'src/mem.cpp', 1, 3, 'void* memory_alloc(size_t n)')",
            [],
        ).unwrap();
        let sym_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id],
        ).unwrap();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, definition, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'buffer_free', 'function', 'void buffer_free()', 'h4', 'src/mem.cpp', 5, 6, 'void buffer_free()')",
            [],
        ).unwrap();
        let sym_id2 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id2],
        ).unwrap();

        fill_symbols_fts(&conn, "test").unwrap();

        // Search "memory alloc" should match memory_alloc
        let hits = search_symbols(&conn, "memory alloc", "test", "main", 10).unwrap();
        assert!(hits.iter().any(|h| h.name == "memory_alloc"));
    }
}
