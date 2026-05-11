use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub id: Option<i64>,
    pub repo: String,
    pub file_path: String,
    pub file_type: String, // "code" or "doc"
    pub summary: String,
    pub content_hash: String,
    pub branch_id: i64,
}

impl FileNode {
    pub fn insert(&self, conn: &Connection, branch_name: &str) -> anyhow::Result<i64> {
        let branch_id = crate::storage::resolve_branch_id(conn, &self.repo, branch_name)?;
        // Build attrs JSON with file_type
        let attrs = serde_json::json!({
            "file_type": self.file_type,
        });
        Ok(conn.query_row(
            "INSERT INTO nodes (repo, node_type, name, content, file_path, content_hash, branch_id, kind, attrs) \
             VALUES (?1, 'file', ?2, ?3, ?2, ?4, ?5, '', ?6) \
             ON CONFLICT(repo, file_path, branch_id) DO UPDATE SET \
             content = excluded.content, content_hash = excluded.content_hash \
             RETURNING id",
            rusqlite::params![
                self.repo, self.file_path, self.summary,
                self.content_hash, branch_id, attrs.to_string()
            ],
            |row| row.get(0),
        )?)
    }
}
