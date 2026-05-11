// Unified node storage — single table for symbols, docs, files
use rusqlite::Connection;
use serde::{Serialize, Deserialize};

/// A universal node — can be a symbol, document section, or file reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub repo: String,
    pub node_type: String,       // "sym" | "doc" | "file"
    pub name: String,
    pub content: String,
    pub file_path: String,
    pub line_start: i64,
    pub content_hash: String,
    pub branch_name: String,
    pub kind: String,            // sym only: function/class/...
    pub attrs: serde_json::Value, // JSON: {signature, namespace, access, level, ...}
}

impl Node {
    pub fn new(repo: &str, node_type: &str, name: &str) -> Self {
        Node {
            id: 0,
            repo: repo.to_string(),
            node_type: node_type.to_string(),
            name: name.to_string(),
            content: String::new(),
            file_path: String::new(),
            line_start: 0,
            content_hash: String::new(),
            branch_name: "main".to_string(),
            kind: String::new(),
            attrs: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Set attrs field from a serializable value
    pub fn set_attr<T: Serialize>(&mut self, key: &str, value: &T) {
        if let serde_json::Value::Object(ref mut map) = self.attrs {
            map.insert(key.to_string(), serde_json::to_value(value).unwrap_or(serde_json::Value::Null));
        }
    }

    /// Get an attr value, returns None if key missing or wrong type
    pub fn get_attr<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.attrs.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Insert a node, returns its auto-assigned ID.
pub fn insert_node(conn: &Connection, node: &Node) -> anyhow::Result<i64> {
    let attrs_str = serde_json::to_string(&node.attrs)?;
    conn.execute(
        "INSERT INTO nodes (repo, node_type, name, content, file_path, line_start, content_hash, branch_name, kind, attrs)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            node.repo, node.node_type, node.name, node.content,
            node.file_path, node.line_start, node.content_hash,
            node.branch_name, node.kind, attrs_str
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get a single node by ID.
pub fn get_node(conn: &Connection, id: i64) -> anyhow::Result<Option<Node>> {
    let mut stmt = conn.prepare(
        "SELECT id, repo, node_type, name, content, file_path, line_start, content_hash, branch_name, kind, attrs
         FROM nodes WHERE id=?1"
    )?;
    let mut rows = stmt.query_map(rusqlite::params![id], |r| {
        let attrs_str: String = r.get(10)?;
        Ok(Node {
            id: r.get(0)?, repo: r.get(1)?, node_type: r.get(2)?,
            name: r.get(3)?, content: r.get(4)?, file_path: r.get(5)?,
            line_start: r.get(6)?, content_hash: r.get(7)?, branch_name: r.get(8)?,
            kind: r.get(9)?,
            attrs: serde_json::from_str(&attrs_str).unwrap_or_default(),
        })
    })?;
    Ok(rows.next().transpose()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    #[test]
    fn test_insert_and_get_node() {
        let conn = Connection::open_in_memory().unwrap();
        storage::migrate(&conn).unwrap();

        let mut node = Node::new("test", "sym", "my_func");
        node.content = "function does stuff".into();
        node.kind = "function".into();
        node.set_attr("signature", &"void my_func()");

        let id = insert_node(&conn, &node).unwrap();
        assert!(id > 0);

        let got = get_node(&conn, id).unwrap().unwrap();
        assert_eq!(got.name, "my_func");
        assert_eq!(got.node_type, "sym");
        assert_eq!(got.kind, "function");
        assert_eq!(got.get_attr::<String>("signature").unwrap(), "void my_func()");
    }
}
