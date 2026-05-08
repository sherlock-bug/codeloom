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
    pub kind: String,     // symbol kind (function/class/enum_value...), empty for docs
    pub sig: String,      // signature for function/method, empty otherwise
    pub snippet: String,  // doc_comment or doc content
}

#[derive(Debug, Clone, PartialEq)]
pub enum HitType {
    CodeName,     // search_symbols_name — name+signature+kind hit
    CodeComment,  // search_symbols_comment — doc_comment hit
    Doc,
    File,
}

/// Fill FTS5 symbol index from symbols table (filtered by repo).
/// Uses 4-column FTS5: name, file_path, signature, kind, doc_comment.
pub fn fill_symbols_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> {
    // Clear old data first (FTS5 doesn't support WHERE DELETE on content tables)
    conn.execute("DELETE FROM fts5_sym", [])?;

    let count = conn.execute(
        "INSERT INTO fts5_sym(rowid, name, file_path, signature, kind, doc_comment)
         SELECT s.rowid, s.name, s.file_path, COALESCE(s.signature, ''), s.kind, COALESCE(s.doc_comment, '') FROM symbols s WHERE s.repo=?1",
        rusqlite::params![repo],
    )?;
    Ok(count)
}

/// Fill FTS5 document index from doc_nodes table (filtered by repo)
pub fn fill_docs_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> {
    conn.execute("DELETE FROM fts5_doc", [])?;

    let count = conn.execute(
        "INSERT INTO fts5_doc(title, section_path, content)
         SELECT COALESCE(title, ''), COALESCE(section_path, ''), content FROM doc_nodes WHERE repo=?1 AND content != '' ORDER BY rowid",
        rusqlite::params![repo],
    )?;
    Ok(count)
}

/// Clear all FTS5 data
pub fn clear_fts(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("DELETE FROM fts5_sym; DELETE FROM fts5_doc; DELETE FROM fts5_files;")?;
    Ok(())
}

/// Fill FTS5 file index from files table (filtered by repo).
pub fn fill_files_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> {
    conn.execute("DELETE FROM fts5_files", [])?;
    let count = conn.execute(
        "INSERT INTO fts5_files(file_path, summary)
         SELECT file_path, summary FROM files WHERE repo=?1 ORDER BY rowid",
        rusqlite::params![repo],
    )?;
    Ok(count)
}

/// FTS5 BM25 keyword search on files (by file_path or summary)
pub fn search_files(
    conn: &Connection,
    query: &str,
    repo: &str,
    limit: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    let safe_query = escape_fts5(query);
    let mut sql = String::from(
        "SELECT fts5_files.rowid, bm25(fts5_files) as score, f.file_path, f.summary
         FROM fts5_files
         JOIN files f ON fts5_files.rowid = f.rowid
         WHERE fts5_files MATCH ?1 AND f.repo = ?2
         ORDER BY score
         LIMIT ?3",
    );
    let mut stmt = conn.prepare(&sql)?;
    let hits = stmt
        .query_map(rusqlite::params![safe_query, repo, limit as i64], |r| {
            let summary: String = r.get(3)?;
            Ok(SearchHit {
                rowid: r.get(0)?,
                score: r.get(1)?,
                hit_type: HitType::File,
                name: r.get(2)?, // file_path as name
                file_path: r.get(2)?,
                line_start: 0,
                kind: String::new(),
                sig: String::new(),
                snippet: summary,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(hits)
}

/// FTS5 BM25 keyword search on symbol names (name + signature + kind columns).
pub fn search_symbols_name(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
) -> anyhow::Result<Vec<SearchHit>> {
    let fts5_query = build_column_query(query, &["name", "signature", "kind"]);
    search_symbols_with_query(conn, &fts5_query, repo, branch, limit, kind_filter, HitType::CodeName)
}

/// FTS5 BM25 keyword search on symbol doc_comment column.
pub fn search_symbols_comment(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
) -> anyhow::Result<Vec<SearchHit>> {
    let fts5_query = build_column_query(query, &["doc_comment"]);
    search_symbols_with_query(conn, &fts5_query, repo, branch, limit, kind_filter, HitType::CodeComment)
}

/// Internal helper: execute FTS5 symbol query with given MATCH string and HitType tag.
fn search_symbols_with_query(
    conn: &Connection,
    fts5_query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
    hit_type: HitType,
) -> anyhow::Result<Vec<SearchHit>> {
    let mut sql = String::from(
        "SELECT fts5_sym.rowid, bm25(fts5_sym) as score, s.name, s.file_path, s.line_start, s.kind, COALESCE(s.doc_comment, ''), COALESCE(s.signature, '')
         FROM fts5_sym
         JOIN symbols s ON s.rowid = fts5_sym.rowid
         JOIN branches b ON b.symbol_id = s.id
         WHERE fts5_sym MATCH ?1 AND s.repo=?2 AND b.branch_name=?3",
    );

    if kind_filter.is_some() {
        sql.push_str(" AND s.kind=?5");
    }

    sql.push_str(" ORDER BY score LIMIT ?4");

    let mut stmt = conn.prepare(&sql)?;

    let hits: Vec<SearchHit> = if let Some(kind) = kind_filter {
        stmt.query_map(
            rusqlite::params![fts5_query, repo, branch, limit as i64, kind],
            |r| map_symbol_hit_with_type(r, &hit_type),
        )?
        .filter_map(|r| r.ok())
        .collect()
    } else {
        stmt.query_map(
            rusqlite::params![fts5_query, repo, branch, limit as i64],
            |r| map_symbol_hit_with_type(r, &hit_type),
        )?
        .filter_map(|r| r.ok())
        .collect()
    };

    Ok(hits)
}

/// Build FTS5 column-filtered query: for column "name" and query "snappy",
/// produces 'name:"snappy" OR name:snappy* OR signature:"snappy" OR ...'
fn build_column_query(query: &str, columns: &[&str]) -> String {
    let terms: Vec<&str> = query.split_whitespace().collect();
    let mut parts = Vec::new();
    for col in columns {
        for &term in &terms {
            let escaped_term = escape_fts5_term(term);
            parts.push(format!("{}:{}", col, escaped_term));
        }
    }
    parts.join(" OR ")
}

/// Escape a single term for FTS5 query (wrapped for use with column: prefix)
fn escape_fts5_term(term: &str) -> String {
    if term.chars().all(|c| c.is_alphanumeric() || c == '_') {
        format!("\"{}\" OR {}*", term, term)
    } else {
        format!("\"{}\"", term)
    }
}

/// Legacy: search_symbols forwards to name search only (backward compat for tests)
pub fn search_symbols(
    conn: &Connection,
    query: &str,
    repo: &str,
    branch: &str,
    limit: usize,
    kind_filter: Option<&str>,
) -> anyhow::Result<Vec<SearchHit>> {
    search_symbols_name(conn, query, repo, branch, limit, kind_filter)
}

fn map_symbol_hit_with_type(r: &rusqlite::Row, hit_type: &HitType) -> rusqlite::Result<SearchHit> {
    let doc_comment: String = r.get(6)?;
    let signature: String = r.get(7)?;
    let snippet = if !doc_comment.is_empty() {
        doc_comment.trim().chars().take(500).collect()
    } else if !signature.is_empty() {
        signature.chars().take(500).collect()
    } else {
        String::new()
    };
    Ok(SearchHit {
        rowid: r.get(0)?,
        score: r.get(1)?,
        hit_type: hit_type.clone(),
        name: r.get(2)?,
        file_path: r.get(3)?,
        line_start: r.get(4)?,
        kind: r.get(5)?,
        sig: signature,
        snippet,
    })
}

/// FTS5 BM25 search on documents only
pub fn search_docs(
    conn: &Connection,
    query: &str,
    repo: &str,
    limit: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    let safe_query = escape_fts5(query);

    let sql = "SELECT fts5_doc.rowid, bm25(fts5_doc) as score, d.title, d.file_path, 0, '', COALESCE(d.content, '')
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
                let content: String = r.get(6)?;
                let snippet: String = content.chars().take(200).collect();
                Ok(SearchHit {
                    rowid: r.get(0)?,
                    score: r.get(1)?,
                    hit_type: HitType::Doc,
                    name: r.get(2)?,
                    file_path: r.get(3)?,
                    line_start: r.get(4)?,
                    kind: String::new(),
                    sig: String::new(),
                    snippet,
                })
            },
        )?
        .filter_map(|r| r.ok())
        .collect();

    Ok(hits)
}

/// Escape FTS5 special characters to prevent query syntax errors.
/// Multi-word queries are joined with AND (each term must appear in the row).
fn escape_fts5(query: &str) -> String {
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.len() == 1 {
        let term = terms[0];
        if term.chars().all(|c| c.is_alphanumeric() || c == '_') {
            format!("\"{}\" OR {}*", term, term)
        } else {
            format!("\"{}\"", term)
        }
    } else {
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
            "INSERT INTO symbols (repo, name, kind, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'AuthService', 'class', 'h1', 'src/auth.cpp', 10, 15, 'class AuthService')",
            [],
        ).unwrap();
        let sym_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id],
        ).unwrap();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'LoginManager', 'class', 'h2', 'src/auth.cpp', 20, 25, 'class LoginManager')",
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
        let hits = search_symbols(&conn, "AuthService", "test", "main", 10, None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "AuthService");
        assert_eq!(hits[0].kind, "class");
        assert!(hits[0].kind == "class", "should have kind=class, got: {}", hits[0].kind);

        // Search partial
        let hits = search_symbols(&conn, "Auth", "test", "main", 10, None).unwrap();
        assert!(hits.iter().any(|h| h.name == "AuthService"));
    }

    #[test]
    fn test_search_kind_filter() {
        let conn = mem_db();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'my_func', 'function', 'h1', 'src/a.cpp', 1, 2, 'void my_func()')",
            [],
        ).unwrap();
        let sid = conn.last_insert_rowid();
        conn.execute("INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')", rusqlite::params![sid]).unwrap();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'MyEnum::A', 'enum_value', 'h2', 'src/b.cpp', 3, 3, 'MyEnum::A')",
            [],
        ).unwrap();
        let sid2 = conn.last_insert_rowid();
        conn.execute("INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')", rusqlite::params![sid2]).unwrap();

        fill_symbols_fts(&conn, "test").unwrap();

        // Without filter — both appear
        let hits = search_symbols(&conn, "my", "test", "main", 10, None).unwrap();
        assert!(hits.iter().any(|h| h.kind == "function"));
        assert!(hits.iter().any(|h| h.kind == "enum_value"));

        // With function filter — only function
        let hits = search_symbols(&conn, "my", "test", "main", 10, Some("function")).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, "function");
        assert_eq!(hits[0].name, "my_func");
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
            "INSERT INTO symbols (repo, name, kind, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'memory_alloc', 'function', 'h3', 'src/mem.cpp', 1, 3, 'void* memory_alloc(size_t n)')",
            [],
        ).unwrap();
        let sym_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id],
        ).unwrap();

        conn.execute(
            "INSERT INTO symbols (repo, name, kind, content_hash, file_path, line_start, line_end, signature)
             VALUES ('test', 'buffer_free', 'function', 'h4', 'src/mem.cpp', 5, 6, 'void buffer_free()')",
            [],
        ).unwrap();
        let sym_id2 = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (symbol_id, repo, branch_name) VALUES (?1, 'test', 'main')",
            rusqlite::params![sym_id2],
        ).unwrap();

        fill_symbols_fts(&conn, "test").unwrap();

        // Search "memory alloc" should match memory_alloc
        let hits = search_symbols(&conn, "memory alloc", "test", "main", 10, None).unwrap();
        assert!(hits.iter().any(|h| h.name == "memory_alloc"));
    }
}
