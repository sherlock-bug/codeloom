//! FTS5 unified full-text search — direct join with nodes table.
//!
//! Architecture:
//!   fts5_all(rowid=node_id, name, content) ←→ nodes(id, ...)
//!
//! Symbol snippet:  n.content (= kind||' '||doc_comment from migration)
//! Signature:       json_extract(n.attrs, '$.signature')
//! Branch filter:   JOIN branches b ON b.node_id = n.id

use rusqlite::Connection;

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub rowid: i64,
    pub score: f64,
    pub hit_type: HitType,
    pub name: String,
    pub file_path: String,
    pub line_start: i64,
    pub kind: String,
    pub sig: String,    // signature (from attrs JSON, symbols only)
    pub snippet: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HitType { Sym, Doc, File }

// ── FTS5 Fill ───────────────────────────────────────────────────────────

/// Rebuild fts5_all from nodes table. Returns number of rows inserted.
pub fn fill_all_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> {
    conn.execute("DELETE FROM fts5_all", [])?;
    let count = conn.execute(
        "INSERT INTO fts5_all(rowid, name, content) SELECT id, name, content FROM nodes WHERE repo=?1",
        rusqlite::params![repo],
    )?;
    Ok(count)
}

/// Migrate old symbols/doc_nodes/files → unified nodes table. Idempotent.
pub fn migrate_to_nodes(conn: &Connection, repo: &str, branch: &str) -> anyhow::Result<usize> {
    // Clear existing nodes + branches for this repo, then re-migrate fresh
    conn.execute("DELETE FROM nodes WHERE repo=?1", rusqlite::params![repo])?;
    conn.execute("DELETE FROM branches WHERE repo=?1", rusqlite::params![repo])?;

    let mut total = 0;

    // Symbols → nodes
    total += conn.execute(
        "INSERT INTO nodes (repo,node_type,name,content,file_path,line_start,content_hash,branch_name,kind,attrs)
         SELECT s.repo,'sym',s.name,
                COALESCE(s.kind,'') || ' ' || COALESCE(s.doc_comment,''),
                s.file_path,s.line_start,s.content_hash,'',s.kind,
                json_object(
                    'signature',COALESCE(s.signature,''),
                    'namespace',COALESCE(s.namespace,''),
                    'access',s.access,
                    'is_virtual',s.is_virtual,
                    'is_definition',s.is_definition,
                    'is_external',s.is_external,
                    'template_args',s.template_args,
                    'language',COALESCE(s.language,''),
                    'parent_class',COALESCE(s.parent_class,''),
                    'line_end',s.line_end
                )
         FROM symbols s WHERE s.repo=?1",
        rusqlite::params![repo],
    )?;

    // Doc_nodes → nodes
    total += conn.execute(
        "INSERT INTO nodes (repo,node_type,name,content,file_path,line_start,content_hash,branch_name,kind,attrs)
         SELECT d.repo,'doc',COALESCE(d.title,''),
                COALESCE(d.section_path,'') || ' ' || d.content,
                d.file_path,0,COALESCE(d.content_hash,''),COALESCE(d.branch_name,'main'),'',
                json_object(
                    'level',d.level,
                    'file_format',COALESCE(d.file_format,''),
                    'section_path',COALESCE(d.section_path,''),
                    'node_type',COALESCE(d.node_type,'section')
                )
         FROM doc_nodes d WHERE d.repo=?1 AND d.content!=''",
        rusqlite::params![repo],
    )?;

    // Files → nodes
    total += conn.execute(
        "INSERT INTO nodes (repo,node_type,name,content,file_path,line_start,content_hash,branch_name,kind,attrs)
         SELECT f.repo,'file',f.file_path,f.summary,
                f.file_path,0,COALESCE(f.content_hash,''),
                COALESCE(f.branch_name,'main'),'',
                json_object('file_type',f.file_type)
         FROM files f WHERE f.repo=?1",
        rusqlite::params![repo],
    )?;

    // Branches: map symbols.id → nodes.id by (name, repo)
    conn.execute(
        "INSERT OR IGNORE INTO branches (node_id, repo, branch_name, override_def, override_hash)
         SELECT n.id, n.repo, ?2, NULL, NULL
         FROM nodes n JOIN symbols s ON s.name=n.name AND s.repo=n.repo
         WHERE n.repo=?1 AND n.node_type='sym'",
        rusqlite::params![repo, branch],
    )?;

    Ok(total)
}

// Backward-compat stubs
pub fn fill_symbols_fts(conn: &Connection, repo: &str) -> anyhow::Result<usize> { fill_all_fts(conn, repo) }
pub fn fill_docs_fts(_c: &Connection, _r: &str) -> anyhow::Result<usize> { Ok(0) }
pub fn fill_files_fts(_c: &Connection, _r: &str) -> anyhow::Result<usize> { Ok(0) }
pub fn clear_fts(conn: &Connection) -> anyhow::Result<()> { conn.execute("DELETE FROM fts5_all", [])?; Ok(()) }

// ── FTS5 Search ─────────────────────────────────────────────────────────

/// Search node name column only (×0.7 weight in fusion layer)
pub fn search_fts5_name(
    conn: &Connection, query: &str, repo: &str, branch: &str,
    limit: usize, kind_filter: Option<&str>,
) -> anyhow::Result<Vec<SearchHit>> {
    let fts5_query = build_column_query(query, &["name"]);
    search_hits(conn, &fts5_query, repo, branch, limit, kind_filter)
}

/// Search node content column only (×0.3 weight in fusion layer)
pub fn search_fts5_content(
    conn: &Connection, query: &str, repo: &str, branch: &str,
    limit: usize, kind_filter: Option<&str>,
) -> anyhow::Result<Vec<SearchHit>> {
    let fts5_query = build_column_query(query, &["content"]);
    search_hits(conn, &fts5_query, repo, branch, limit, kind_filter)
}

// Backward-compat wrappers (keep external callers working)
pub fn search_symbols(
    conn: &Connection, query: &str, repo: &str, branch: &str, limit: usize, kf: Option<&str>
) -> anyhow::Result<Vec<SearchHit>> {
    search_fts5_name(conn, query, repo, branch, limit, kf)
}
pub fn search_docs(
    conn: &Connection, query: &str, repo: &str, limit: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    search_fts5_content(conn, query, repo, "main", limit, None)
}
pub fn search_files(
    conn: &Connection, query: &str, repo: &str, limit: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    search_fts5_name(conn, query, repo, "main", limit, None)
}

// ── Internal ────────────────────────────────────────────────────────────

/// Core unified search: sym + doc + file from fts5_all, merged and sorted.
fn search_hits(
    conn: &Connection, fts5_query: &str, repo: &str, branch: &str,
    limit: usize, kind_filter: Option<&str>,
) -> anyhow::Result<Vec<SearchHit>> {
    let kind_val = kind_filter.unwrap_or("");
    let mut hits: Vec<SearchHit> = Vec::new();

    // ── Symbols ──────────────────────────────────────────────────────
    {
        let sql = "\
            SELECT fts5_all.rowid, bm25(fts5_all) as score,
                   n.name, n.file_path, n.line_start, n.kind,
                   n.content, json_extract(n.attrs,'$.signature')
            FROM fts5_all
            JOIN nodes n ON n.id = fts5_all.rowid
            JOIN branches b ON b.node_id = n.id
            WHERE fts5_all MATCH ?1
              AND n.repo = ?2
              AND b.branch_name = ?3
              AND n.node_type = 'sym'
              AND (n.kind = ?5 OR ?5 = '')
            ORDER BY score
            LIMIT ?4";

        let mut stmt = conn.prepare(sql)?;
        let sym_hits: Vec<SearchHit> = stmt.query_map(
            rusqlite::params![fts5_query, repo, branch, limit as i64, kind_val],
            |r| {
                let content: String = r.get(6)?;  // n.content = "kind doc_comment"
                let sig: String = r.get::<_, Option<String>>(7)?.unwrap_or_default(); // attrs->signature, may be NULL
                // Strip kind prefix from content for snippet
                let kind: String = r.get(5)?;
                let snip = strip_kind_prefix(&content, &kind);
                let sig_short: String = sig.chars().take(500).collect();
                Ok(SearchHit {
                    rowid: r.get(0)?, score: r.get(1)?, hit_type: HitType::Sym,
                    name: r.get(2)?, file_path: r.get(3)?, line_start: r.get(4)?,
                    kind, sig: sig_short, snippet: snip,
                })
            },
        )?.filter_map(|r| r.ok()).collect();
        hits.extend(sym_hits);
    }

    // ── Docs ─────────────────────────────────────────────────────────
    {
        let sql = "\
            SELECT fts5_all.rowid, bm25(fts5_all) as score,
                   n.name, n.file_path, n.content
            FROM fts5_all
            JOIN nodes n ON n.id = fts5_all.rowid
            WHERE fts5_all MATCH ?1
              AND n.repo = ?2
              AND n.node_type = 'doc'
            ORDER BY score
            LIMIT ?3";

        let mut stmt = conn.prepare(sql)?;
        let doc_hits: Vec<SearchHit> = stmt.query_map(
            rusqlite::params![fts5_query, repo, limit as i64],
            |r| {
                let content: String = r.get(4)?;
                let snip: String = content.chars().take(200).collect();
                let title: String = r.get(2)?;
                let name = if title.is_empty() {
                    content.chars().take(60).collect::<String>().trim().to_string()
                } else { title };
                Ok(SearchHit {
                    rowid: r.get(0)?, score: r.get(1)?, hit_type: HitType::Doc,
                    name, file_path: r.get(3)?, line_start: 0,
                    kind: String::new(), sig: String::new(), snippet: snip,
                })
            },
        )?.filter_map(|r| r.ok()).collect();
        hits.extend(doc_hits);
    }

    // ── Files ────────────────────────────────────────────────────────
    {
        let sql = "\
            SELECT fts5_all.rowid, bm25(fts5_all) as score,
                   n.name, n.file_path, n.content
            FROM fts5_all
            JOIN nodes n ON n.id = fts5_all.rowid
            WHERE fts5_all MATCH ?1
              AND n.repo = ?2
              AND n.node_type = 'file'
            ORDER BY score
            LIMIT ?3";

        let mut stmt = conn.prepare(sql)?;
        let file_hits: Vec<SearchHit> = stmt.query_map(
            rusqlite::params![fts5_query, repo, limit as i64],
            |r| {
                let summary: String = r.get(4)?;
                Ok(SearchHit {
                    rowid: r.get(0)?, score: r.get(1)?, hit_type: HitType::File,
                    name: r.get(2)?, file_path: r.get(3)?, line_start: 0,
                    kind: String::new(), sig: String::new(), snippet: summary,
                })
            },
        )?.filter_map(|r| r.ok()).collect();
        hits.extend(file_hits);
    }

    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hits.truncate(limit);
    Ok(hits)
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Build FTS5 column-prefixed query: "name:\"term\" OR name:term*" for each term.
fn build_column_query(query: &str, columns: &[&str]) -> String {
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.is_empty() { return format!("\"{}\"", query); }
    let mut parts = Vec::new();
    for col in columns {
        for &term in &terms {
            parts.push(format!("{}:\"{}\"", col, term));
            if term.chars().all(|c| c.is_alphanumeric() || c == '_') {
                parts.push(format!("{}:{}*", col, term));
            }
        }
    }
    parts.join(" OR ")
}

/// Strip "kind " prefix from content to get clean doc_comment snippet.
/// e.g. "class This is a comment" → "This is a comment"
fn strip_kind_prefix(content: &str, kind: &str) -> String {
    if kind.is_empty() { return content.chars().take(500).collect(); }
    let prefix = format!("{} ", kind);
    let s = if content.starts_with(&prefix) {
        &content[prefix.len()..]
    } else {
        content
    };
    s.trim().chars().take(500).collect()
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    fn mem_db() -> (Connection, String) {
        let conn = Connection::open_in_memory().unwrap();
        storage::schema::run(&conn).unwrap();
        let repo = "test";
        let branch = "main";
        // Insert a symbol node
        conn.execute(
            "INSERT INTO nodes (repo,node_type,name,content,content_hash,file_path,kind) \
             VALUES (?1,'sym','AuthService','class Handles authentication','h1','src/auth.cpp','class')",
            rusqlite::params![repo],
        ).unwrap();
        let nid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO branches (node_id,repo,branch_name) VALUES (?1,?2,?3)",
            rusqlite::params![nid, repo, branch],
        ).unwrap();
        // Insert a doc node
        conn.execute(
            "INSERT INTO nodes (repo,node_type,name,content,content_hash,file_path) \
             VALUES (?1,'doc','Getting Started','Installation guide content here','h2','docs/start.md')",
            rusqlite::params![repo],
        ).unwrap();
        fill_all_fts(&conn, repo).unwrap();
        (conn, repo.to_string())
    }

    #[test]
    fn test_search_symbols() {
        let (conn, repo) = mem_db();
        let hits = search_symbols(&conn, "AuthService", &repo, "main", 10, None).unwrap();
        let fts5_query = "name:\"AuthService\" OR name:AuthService*";
        let sql = "SELECT fts5_all.rowid, n.name FROM fts5_all JOIN nodes n ON n.id = fts5_all.rowid JOIN branches b ON b.node_id = n.id WHERE fts5_all MATCH ?1 AND n.repo = ?2 AND b.branch_name = ?3 AND n.node_type = 'sym'";
        let rows: Vec<_> = conn.prepare(sql).unwrap()
            .query_map(rusqlite::params![fts5_query, "test", "main"], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))).unwrap().flatten().collect();
        eprintln!("DEBUG exact SQL: {:?}", rows);
        let hits = search_symbols(&conn, "AuthService", &repo, "main", 10, None).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "AuthService");
        assert_eq!(hits[0].kind, "class");
        assert_eq!(hits[0].snippet, "Handles authentication");
    }

    #[test]
    fn test_search_docs() {
        let (conn, repo) = mem_db();
        let hits = search_docs(&conn, "Installation", &repo, 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "Getting Started");
    }

    #[test]
    fn test_strip_kind_prefix() {
        assert_eq!(strip_kind_prefix("class Handles auth", "class"), "Handles auth");
        assert_eq!(strip_kind_prefix("Handles auth", ""), "Handles auth");
        assert_eq!(strip_kind_prefix("", ""), "");
    }
}
