use rusqlite::Connection;
pub mod dedup; pub mod fts; pub mod schema; pub mod symbols; pub mod vector;

pub fn open(path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}
pub fn migrate(conn: &Connection) -> anyhow::Result<()> { schema::run(conn) }

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn test_open_in_memory() {
        let conn = Connection::open_in_memory();
        assert!(conn.is_ok());
    }

    #[test]
    fn test_migrate_creates_tables() {
        let conn = mem_db();
        // Check all tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"symbols".to_string()));
        assert!(tables.contains(&"edges".to_string()));
        assert!(tables.contains(&"branches".to_string()));
        assert!(tables.contains(&"doc_nodes".to_string()));
        assert!(tables.contains(&"git_index_state".to_string()));
        assert!(tables.contains(&"branch_glossary".to_string()));
    }

    #[test]
    fn test_hash_content_deterministic() {
        let h1 = dedup::hash_content("hello");
        let h2 = dedup::hash_content("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_content_different() {
        let h1 = dedup::hash_content("hello");
        let h2 = dedup::hash_content("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_symbol_insert_and_query() {
        let conn = mem_db();
        let sym = symbols::Symbol {
            id: None,
            repo: "test".into(),
            name: "my_func".into(),
            kind: "function".into(),
            definition: "void my_func() {}".into(),
            content_hash: dedup::hash_content("void my_func() {}"),
            file_path: "src/main.cpp".into(),
            line_start: 10,
            line_end: 12,
            language: Some("cpp".into()),
            signature: Some("void my_func()".into()),
            parent_class: None,
            namespace: None,
        };
        let id = sym.insert(&conn).unwrap();
        assert!(id > 0);

        let name: String = conn
            .query_row("SELECT name FROM symbols WHERE id=?1", rusqlite::params![id], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "my_func");
    }

    #[test]
    fn test_symbol_on_conflict_no_error() {
        let conn = mem_db();
        let sym = symbols::Symbol {
            id: None,
            repo: "test".into(),
            name: "dup".into(),
            kind: "function".into(),
            definition: "void dup() {}".into(),
            content_hash: "abc".into(),
            file_path: "x.cpp".into(),
            line_start: 1,
            line_end: 1,
            language: None,
            signature: None,
            parent_class: None,
            namespace: None,
        };
        let id1 = sym.insert(&conn).unwrap();
        let id2 = sym.insert(&conn).unwrap();
        assert_eq!(id1, id2); // ON CONFLICT returns same id
    }
}
