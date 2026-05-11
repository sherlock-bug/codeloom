// sqlite-vec vector storage — statically compiled, no .so needed
use rusqlite::Connection;
use crate::log_warn;

/// Verify vec0 is available (statically compiled in).
pub fn try_load(conn: &Connection) -> bool {
    let ok = conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS _vec0_test_ USING vec0(embedding FLOAT[1]);
         DROP TABLE IF EXISTS _vec0_test_;",
    )
    .is_ok();
    if !ok {
        log_warn!("vector", "vec0 not loaded: sqlite-vec extension unavailable");
    }
    ok
}

/// Create vec0 virtual tables (FLOAT32) for this repo.
pub fn create_tables(conn: &Connection, repo: &str, dim: usize) -> anyhow::Result<()> {
    let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    conn.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {sym_name_table} USING vec0(embedding FLOAT[{dim}] distance_metric=cosine);
         CREATE VIRTUAL TABLE IF NOT EXISTS {file_table} USING vec0(embedding FLOAT[{dim}]);"
    ))?;
    Ok(())
}

/// Clear all vectors for a repo (drop & recreate to avoid vec0 UNIQUE conflicts).
pub fn clear_vectors(conn: &Connection, repo: &str, dim: usize) -> anyhow::Result<()> {
    let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    let _ = conn.execute_batch(&format!(
        "DROP TABLE IF EXISTS {sym_name_table}; DROP TABLE IF EXISTS {file_table};"
    ));
    create_tables(conn, repo, dim)
}

/// Bulk insert FLOAT32 vectors. Vectors are JSON float arrays.
pub fn insert_vectors(conn: &Connection, table: &str, rows: &[(i64, &[f32])]) -> anyhow::Result<usize> {
    let mut count = 0;
    for &(rowid, vec) in rows {
        let json = format!("[{}]", vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        conn.execute(
            &format!("INSERT OR IGNORE INTO {table} (rowid, embedding) VALUES (?1, ?2)"),
            rusqlite::params![rowid, json],
        )?;
        count += 1;
    }
    Ok(count)
}

/// Execute a vec0 KNN query. Returns (rowid, distance) pairs.
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
