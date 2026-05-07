use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub id: Option<i64>,
    pub repo: String,
    pub file_path: String,
    pub file_type: String, // "code" or "doc"
    pub summary: String,
    pub content_hash: String,
    pub branch_name: String,
}

impl FileNode {
    pub fn insert(&self, conn: &Connection) -> anyhow::Result<i64> {
        Ok(conn.query_row(
            "INSERT INTO files (repo, file_path, file_type, summary, content_hash, branch_name) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(repo, file_path, branch_name) DO UPDATE SET \
             summary = excluded.summary, content_hash = excluded.content_hash \
             RETURNING id",
            rusqlite::params![
                self.repo, self.file_path, self.file_type,
                self.summary, self.content_hash, self.branch_name
            ],
            |row| row.get(0),
        )?)
    }
}
