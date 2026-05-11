use rusqlite::Connection;
pub mod dedup; pub mod files; pub mod fts; pub mod nodes; pub mod schema; pub mod symbols; pub mod vector;

pub fn open(path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> anyhow::Result<()> {
    schema::run(conn)?;
    Ok(())
}

/// Resolve a (repo, branch_name) pair to a branch_meta.id.
/// Creates the entry if it doesn't exist.
pub fn resolve_branch_id(conn: &Connection, repo: &str, branch_name: &str) -> anyhow::Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO branch_meta (repo, branch_name) VALUES (?1, ?2)",
        rusqlite::params![repo, branch_name],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM branch_meta WHERE repo=?1 AND branch_name=?2",
        rusqlite::params![repo, branch_name],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Insert document images extracted during doc indexing.
pub fn insert_doc_images(
    conn: &Connection,
    doc_node_id: i64,
    images: &[crate::doc::section::ImageRef],
) -> anyhow::Result<()> {
    // Normalize: if raw_bytes is empty (linked image), store original_src only
    for img in images {
        let original_size = img.raw_bytes.len() as i64;
        conn.execute(
            "INSERT INTO doc_images (doc_node_id, alt_text, original_src, image_data, position, section_context, image_type, original_size) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                doc_node_id,
                img.alt_text,
                img.original_src,
                img.raw_bytes,
                img.position,
                img.section_context,
                img.image_type,
                original_size,
            ],
        )?;
    }
    Ok(())
}
