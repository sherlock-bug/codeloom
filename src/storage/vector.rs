// sqlite-vec vector storage — loadable extension for ANN vector search
use rusqlite::Connection;

/// Try to load the vec0 extension. Returns Ok(true) if loaded, Ok(false) if not found.
pub fn try_load(conn: &Connection) -> bool {
    // Look for vec0.so alongside the binary, then in models/
    let candidates = [
        "models/sqlite-vec/vec0.so".to_string(),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("models/sqlite-vec/vec0.so")
            .to_string_lossy()
            .to_string(),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            // Enable extension loading via unsafe API
            unsafe {
                if conn.load_extension_enable().is_err() { return false; }
                if conn.load_extension(path, None).is_ok() {
                    return true;
                }
                let _ = conn.load_extension_disable();
            }
            return false;
        }
    }
    false
}

/// Create vec0 virtual tables for this repo (symbol vectors + doc vectors)
pub fn create_tables(conn: &Connection, repo: &str) -> anyhow::Result<()> {
    let sym_table = format!("symbol_vec_{}", repo.replace('-', "_"));
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));
    
    conn.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {sym_table} USING vec0(embedding FLOAT[512]);
         CREATE VIRTUAL TABLE IF NOT EXISTS {doc_table} USING vec0(embedding FLOAT[512]);"
    ))?;
    Ok(())
}

/// Bulk insert vectors into a vec0 table. Vectors are JSON arrays.
pub fn insert_vectors(conn: &Connection, table: &str, rows: &[(i64, &[f32])]) -> anyhow::Result<usize> {
    let mut count = 0;
    for &(rowid, vec) in rows {
        let json = format!("[{}]", vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        conn.execute(
            &format!("INSERT OR REPLACE INTO {table} (rowid, embedding) VALUES (?1, ?2)"),
            rusqlite::params![rowid, json],
        )?;
        count += 1;
    }
    Ok(count)
}

/// Execute a vec0 KNN query. Returns (rowid, distance) pairs sorted by distance.
pub fn knn_search(conn: &Connection, table: &str, query_vec: &[f32], k: usize) -> anyhow::Result<Vec<(i64, f64)>> {
    let json = format!("[{}]", query_vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    let sql = format!("SELECT rowid, distance FROM {table} WHERE embedding MATCH ?1 ORDER BY distance LIMIT ?2");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![json, k as i64], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
