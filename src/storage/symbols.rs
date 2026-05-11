use rusqlite::Connection;
use sha2::{Digest, Sha256};
use crate::log_debug;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: Option<i64>,
    pub repo: String,
    pub name: String,
    pub kind: String,
    pub content_hash: String,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub language: Option<String>,
    pub signature: Option<String>,
    pub parent_class: Option<String>,
    pub namespace: Option<String>,
    pub doc_comment: String,
    pub sid: Option<String>,
    pub access: String,
    pub is_virtual: bool,
    pub is_definition: bool,
    pub is_external: bool,
    pub template_args: Option<String>,
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol {
            id: None,
            repo: String::new(),
            name: String::new(),
            kind: String::new(),
            content_hash: String::new(),
            file_path: String::new(),
            line_start: 0,
            line_end: 0,
            language: None,
            signature: None,
            parent_class: None,
            namespace: None,
            doc_comment: String::new(),
            sid: None,
            access: String::new(),
            is_virtual: false,
            is_definition: true,
            is_external: false,
            template_args: None,
        }
    }
}

impl Symbol {
    /// Insert symbol into `nodes` table (node_type='sym').
    /// Returns the new nodes.id.
    pub fn insert(&self, conn: &Connection, branch_name: &str) -> anyhow::Result<i64> {
        // Resolve branch name to ID
        let branch_id = crate::storage::resolve_branch_id(conn, &self.repo, branch_name)?;
        // Auto-generate sid if not provided
        let sid = self.sid.clone().unwrap_or_else(|| {
            let sig = self.signature.as_deref().unwrap_or("");
            let ns = self.namespace.as_deref().unwrap_or("");
            make_sid(&self.repo, &self.name, sig, ns, &self.kind, &self.file_path)
        });

        // Build content = "kind doc_comment" (for FTS5 search)
        let content = if self.doc_comment.is_empty() {
            self.kind.clone()
        } else {
            format!("{} {}", self.kind, self.doc_comment)
        };

        // Build attrs JSON
        let attrs = serde_json::json!({
            "signature": self.signature,
            "namespace": self.namespace,
            "access": self.access,
            "is_virtual": self.is_virtual,
            "is_definition": self.is_definition,
            "is_external": self.is_external,
            "template_args": self.template_args,
            "language": self.language,
            "parent_class": self.parent_class,
            "line_end": self.line_end,
            "sid": sid,
        });

        // Check for existing node with same content_hash + file_path + name + repo
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM nodes WHERE content_hash=?1 AND file_path=?2 AND name=?3 AND repo=?4 AND node_type='sym'",
                rusqlite::params![self.content_hash, self.file_path, self.name, self.repo],
                |row| row.get(0),
            )
            .ok();

        if let Some(nid) = existing {
            // Update: line_start, line_end, doc_comment (append), attrs for specific fields
            conn.execute(
                "UPDATE nodes SET line_start=?1, line_end=?2, \
                 content = CASE WHEN ?3 != '' AND content NOT LIKE '%' || ?3 || '%' \
                   THEN content || CHAR(10) || ?3 ELSE content END \
                 WHERE id=?4",
                rusqlite::params![self.line_start, self.line_end, self.doc_comment, nid],
            )?;
            // Update attrs: sid, is_definition, is_external
            conn.execute(
                "UPDATE nodes SET attrs = json_set(attrs, '$.is_definition', ?1, '$.is_external', ?2) WHERE id=?3",
                rusqlite::params![self.is_definition as i32, self.is_external as i32, nid],
            )?;
            log_debug!("symbols::insert", "upsert: name={} kind={} action=update", self.name, self.kind);
            Ok(nid)
        } else {
            let nid = conn.query_row(
                "INSERT INTO nodes (repo,node_type,name,content,file_path,line_start,content_hash,branch_id,kind,attrs) \
                 VALUES (?1,'sym',?2,?3,?4,?5,?6,?7,?8,?9) RETURNING id",
                rusqlite::params![
                    self.repo, self.name, content, self.file_path, self.line_start,
                    self.content_hash, branch_id, self.kind, attrs.to_string(),
                ],
                |row| row.get(0),
            )?;
            log_debug!("symbols::insert", "upsert: name={} kind={} action=insert", self.name, self.kind);
            Ok(nid)
        }
    }
}

/// Insert builtin C++ standard library symbols as synthetic nodes.
pub fn insert_builtin_symbols(conn: &Connection) -> anyhow::Result<usize> {
    let repo = "__builtin__";
    let mut count = 0;

    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM nodes WHERE repo=?1 AND node_type='sym'",
            rusqlite::params![repo],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if existing > 0 {
        return Ok(0);
    }

    let containers: &[(&str, &[&str])] = &[
        (
            "std::vector",
            &[
                "push_back", "pop_back", "size", "empty", "clear", "begin", "end", "at",
                "front", "back", "insert", "erase", "emplace_back", "reserve", "capacity",
                "data", "resize", "shrink_to_fit",
            ],
        ),
        (
            "std::string",
            &[
                "find", "substr", "size", "length", "c_str", "append", "clear", "empty",
                "data", "push_back", "pop_back", "reserve", "capacity", "compare", "replace",
                "erase", "insert", "begin", "end", "resize",
            ],
        ),
        (
            "std::map",
            &[
                "find", "insert", "erase", "size", "empty", "clear", "begin", "end", "at",
                "count", "lower_bound", "upper_bound", "emplace", "try_emplace", "extract",
                "merge",
            ],
        ),
        (
            "std::unordered_map",
            &[
                "find", "insert", "erase", "size", "empty", "clear", "begin", "end", "at",
                "count", "emplace", "try_emplace", "extract", "bucket", "load_factor",
                "rehash", "reserve",
            ],
        ),
        (
            "std::set",
            &[
                "find", "insert", "erase", "size", "empty", "clear", "begin", "end", "count",
                "lower_bound", "upper_bound", "emplace", "extract",
            ],
        ),
        (
            "std::unordered_set",
            &[
                "find", "insert", "erase", "size", "empty", "clear", "begin", "end", "count",
                "emplace", "extract",
            ],
        ),
        (
            "std::unique_ptr",
            &[
                "get", "reset", "release", "operator bool", "operator*", "operator->",
            ],
        ),
        (
            "std::shared_ptr",
            &[
                "get", "reset", "use_count", "unique", "operator bool", "operator*",
                "operator->",
            ],
        ),
    ];

    let free_funcs: &[&str] = &[
        "std::find", "std::find_if", "std::count", "std::count_if", "std::sort",
        "std::stable_sort", "std::copy", "std::copy_if", "std::transform", "std::for_each",
        "std::lower_bound", "std::upper_bound", "std::binary_search", "std::equal_range",
        "std::min", "std::max", "std::min_element", "std::max_element", "std::swap",
        "std::reverse", "std::rotate", "std::fill", "std::generate", "std::remove",
        "std::remove_if", "std::unique", "std::move", "std::forward", "std::make_unique",
        "std::make_shared", "std::make_pair", "std::begin", "std::end",
    ];

    for &(container, methods) in containers {
        for &method in methods {
            let name = format!("{}::{}", container, method);
            let content_hash = format!("builtin:{}", count);
            let sid = make_sid(repo, &name, "", "std", "method", "");
            let attrs = serde_json::json!({"sid": sid, "namespace": "std", "language": "cpp"});
            conn.execute(
                "INSERT OR IGNORE INTO nodes (repo,node_type,name,content,file_path,content_hash,kind,attrs) \
                 VALUES (?1,'sym',?2,'method','',?3,'method',?4)",
                rusqlite::params![repo, name, content_hash, attrs.to_string()],
            )?;
            count += 1;
        }
    }

    for &func in free_funcs {
        let content_hash = format!("builtin:{}", count);
        let sid = make_sid(repo, func, "", "std", "function", "");
        let attrs = serde_json::json!({"sid": sid, "namespace": "std", "language": "cpp"});
        conn.execute(
            "INSERT OR IGNORE INTO nodes (repo,node_type,name,content,file_path,content_hash,kind,attrs) \
             VALUES (?1,'sym',?2,'function','',?3,'function',?4)",
            rusqlite::params![repo, func, content_hash, attrs.to_string()],
        )?;
        count += 1;
    }

    // Populate branches from nodes table
    conn.execute(
        "INSERT OR IGNORE INTO branches (node_id,repo,branch_id,branch_name) \
         SELECT id,repo,0,'__builtin__' FROM nodes WHERE repo=?1 AND node_type='sym'",
        rusqlite::params![repo],
    )?;

    Ok(count)
}

/// Generate stable ID from key fields
pub fn make_sid(
    repo: &str,
    name: &str,
    sig: &str,
    ns: &str,
    kind: &str,
    file_path: &str,
) -> String {
    let mut h = Sha256::new();
    h.update(repo.as_bytes());
    h.update(name.as_bytes());
    h.update(sig.as_bytes());
    h.update(ns.as_bytes());
    h.update(kind.as_bytes());
    h.update(file_path.as_bytes());
    format!("{:x}", h.finalize()).chars().take(16).collect()
}
