use rusqlite::Connection;
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: Option<i64>, pub repo: String, pub name: String, pub kind: String,
    pub content_hash: String, pub file_path: String,
    pub line_start: u32, pub line_end: u32, pub language: Option<String>,
    pub signature: Option<String>, pub parent_class: Option<String>, pub namespace: Option<String>,
    pub doc_comment: String,
}
impl Symbol {
    pub fn insert(&self, conn: &Connection) -> anyhow::Result<i64> {
        Ok(conn.query_row(
            "INSERT INTO symbols (repo,name,kind,content_hash,file_path,line_start,line_end,language,signature,parent_class,namespace,doc_comment) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) \
             ON CONFLICT(content_hash,file_path,name,repo) DO UPDATE SET \
             line_start=excluded.line_start, line_end=excluded.line_end, \
             doc_comment = CASE WHEN excluded.doc_comment != '' THEN \
               CASE WHEN doc_comment != '' THEN doc_comment || '\n' || excluded.doc_comment \
               ELSE excluded.doc_comment END \
             ELSE doc_comment END \
             RETURNING id",
            rusqlite::params![self.repo,self.name,self.kind,self.content_hash,self.file_path,self.line_start,self.line_end,self.language,self.signature,self.parent_class,self.namespace,self.doc_comment],
            |row| row.get(0),
        )?)
    }
}