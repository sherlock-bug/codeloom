use rusqlite::Connection;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: Option<i64>, pub repo: String, pub name: String, pub kind: String,
    pub content_hash: String, pub file_path: String,
    pub line_start: u32, pub line_end: u32, pub language: Option<String>,
    pub signature: Option<String>, pub parent_class: Option<String>, pub namespace: Option<String>,
    pub doc_comment: String,
    pub sid: Option<String>,
    pub access: String, pub is_virtual: bool, pub is_definition: bool,
    pub is_external: bool, pub template_args: Option<String>,
}

impl Default for Symbol {
    fn default() -> Self {
        Symbol {
            id: None, repo: String::new(), name: String::new(), kind: String::new(),
            content_hash: String::new(), file_path: String::new(),
            line_start: 0, line_end: 0, language: None,
            signature: None, parent_class: None, namespace: None,
            doc_comment: String::new(),
            sid: None, access: String::new(), is_virtual: false,
            is_definition: true, is_external: false, template_args: None,
        }
    }
}

impl Symbol {
    pub fn insert(&self, conn: &Connection) -> anyhow::Result<i64> {
        // Auto-generate sid if not provided: SHA256(repo+name+sig+ns+kind)
        let sid = self.sid.clone().unwrap_or_else(|| {
            let sig = self.signature.as_deref().unwrap_or("");
            let ns = self.namespace.as_deref().unwrap_or("");
            make_sid(&self.repo, &self.name, sig, ns, &self.kind, &self.file_path)
        });

        Ok(conn.query_row(
            "INSERT INTO symbols (repo,name,kind,content_hash,file_path,line_start,line_end,language,signature,parent_class,namespace,doc_comment,sid,access,is_virtual,is_definition,is_external,template_args) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18) \
             ON CONFLICT(content_hash,file_path,name,repo) DO UPDATE SET \
             line_start=excluded.line_start, line_end=excluded.line_end, \
             doc_comment = CASE WHEN excluded.doc_comment != '' THEN \
               CASE WHEN doc_comment != '' THEN doc_comment || CHAR(10) || excluded.doc_comment \
               ELSE excluded.doc_comment END \
             ELSE doc_comment END \
             RETURNING id",
            rusqlite::params![
                self.repo, self.name, self.kind, self.content_hash, self.file_path,
                self.line_start, self.line_end, self.language, self.signature,
                self.parent_class, self.namespace, self.doc_comment,
                sid, self.access, self.is_virtual as i32, self.is_definition as i32,
                self.is_external as i32, self.template_args,
            ],
            |row| row.get(0),
        )?)
    }
}

/// Insert builtin C++ standard library symbols as synthetic nodes.
pub fn insert_builtin_symbols(conn: &Connection) -> anyhow::Result<usize> {
    let repo = "__builtin__";
    let mut count = 0;

    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM symbols WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)
    ).unwrap_or(0);
    if existing > 0 { return Ok(0); }

    let containers: &[(&str, &[&str])] = &[
        ("std::vector", &["push_back","pop_back","size","empty","clear","begin","end","at","front","back","insert","erase","emplace_back","reserve","capacity","data","resize","shrink_to_fit"]),
        ("std::string", &["find","substr","size","length","c_str","append","clear","empty","data","push_back","pop_back","reserve","capacity","compare","replace","erase","insert","begin","end","resize"]),
        ("std::map", &["find","insert","erase","size","empty","clear","begin","end","at","count","lower_bound","upper_bound","emplace","try_emplace","extract","merge"]),
        ("std::unordered_map", &["find","insert","erase","size","empty","clear","begin","end","at","count","emplace","try_emplace","extract","bucket","load_factor","rehash","reserve"]),
        ("std::set", &["find","insert","erase","size","empty","clear","begin","end","count","lower_bound","upper_bound","emplace","extract"]),
        ("std::unordered_set", &["find","insert","erase","size","empty","clear","begin","end","count","emplace","extract"]),
        ("std::unique_ptr", &["get","reset","release","operator bool","operator*","operator->"]),
        ("std::shared_ptr", &["get","reset","use_count","unique","operator bool","operator*","operator->"]),
    ];

    let free_funcs: &[&str] = &[
        "std::find","std::find_if","std::count","std::count_if","std::sort","std::stable_sort",
        "std::copy","std::copy_if","std::transform","std::for_each",
        "std::lower_bound","std::upper_bound","std::binary_search","std::equal_range",
        "std::min","std::max","std::min_element","std::max_element",
        "std::swap","std::reverse","std::rotate","std::fill","std::generate",
        "std::remove","std::remove_if","std::unique",
        "std::move","std::forward",
        "std::make_unique","std::make_shared","std::make_pair",
        "std::begin","std::end",
    ];

    for &(container, methods) in containers {
        for &method in methods {
            let name = format!("{}::{}", container, method);
            let content_hash = format!("builtin:{}", count);
            let sid = make_sid(repo, &name, "", "std", "method", "");
            conn.execute(
                "INSERT OR IGNORE INTO symbols (repo,name,kind,content_hash,file_path,line_start,line_end,language,doc_comment,sid,namespace,is_external) \
                 VALUES (?1,?2,'method',?3,'',0,0,'cpp','',?4,'std',0)",
                rusqlite::params![repo, name, content_hash, sid],
            )?;
            count += 1;
        }
    }

    for &func in free_funcs {
        let content_hash = format!("builtin:{}", count);
        let sid = make_sid(repo, func, "", "std", "function", "");
        conn.execute(
            "INSERT OR IGNORE INTO symbols (repo,name,kind,content_hash,file_path,line_start,line_end,language,doc_comment,sid,namespace,is_external) \
             VALUES (?1,?2,'function',?3,'',0,0,'cpp','',?4,'std',0)",
            rusqlite::params![repo, func, content_hash, sid],
        )?;
        count += 1;
    }

    conn.execute(
        "INSERT OR IGNORE INTO branches (symbol_id,repo,branch_name) \
         SELECT id,repo,NULL FROM symbols WHERE repo=?1",
        rusqlite::params![repo],
    )?;

    Ok(count)
}

/// Generate stable ID from key fields
pub fn make_sid(repo: &str, name: &str, sig: &str, ns: &str, kind: &str, file_path: &str) -> String {
    let mut h = Sha256::new();
    h.update(repo.as_bytes());
    h.update(name.as_bytes());
    h.update(sig.as_bytes());
    h.update(ns.as_bytes());
    h.update(kind.as_bytes());
    h.update(file_path.as_bytes());
    format!("{:x}", h.finalize()).chars().take(16).collect()
}
