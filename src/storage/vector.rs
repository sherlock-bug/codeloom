// sqlite-vec vector storage — statically compiled, no .so needed
use rusqlite::Connection;

/// Verify vec0 is available (statically compiled in).
/// Returns true if vec0 virtual tables can be created.
pub fn try_load(conn: &Connection) -> bool {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS _vec0_test_ USING vec0(embedding FLOAT[1]);
         DROP TABLE IF EXISTS _vec0_test_;",
    )
    .is_ok()
}

/// Create vec0 virtual tables for this repo (symbol name vectors INT8 + file vectors FLOAT32)
pub fn create_tables(conn: &Connection, repo: &str, dim: usize) -> anyhow::Result<()> {
    let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));

    conn.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {sym_name_table} USING vec0(embedding INT8[{dim}] distance_metric=cosine);
         CREATE VIRTUAL TABLE IF NOT EXISTS {file_table} USING vec0(embedding INT8[{dim}]);"
    ))?;
    Ok(())
}

/// Clear all vectors for a repo (before re-indexing to avoid vec0 UNIQUE conflicts)
pub fn clear_vectors(conn: &Connection, repo: &str, dim: usize) -> anyhow::Result<()> {
    let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    // vec0 virtual tables don't support DELETE with WHERE, so drop and recreate
    let _ = conn.execute_batch(&format!(
        "DROP TABLE IF EXISTS {sym_name_table}; DROP TABLE IF EXISTS {file_table};"
    ));
    create_tables(conn, repo, dim)
}

/// Bulk insert float32 vectors into a vec0 FLOAT table. Vectors are JSON float arrays.
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

/// Bulk insert int8 vectors into a vec0 INT8 table. Vectors are JSON integer arrays.
pub fn insert_vectors_int8(conn: &Connection, table: &str, rows: &[(i64, &[i8])]) -> anyhow::Result<usize> {
    let mut count = 0;
    for &(rowid, vec) in rows {
        let json = format!("[{}]", vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        conn.execute(
            &format!("INSERT OR IGNORE INTO {table} (rowid, embedding) VALUES (?1, vec_int8(?2))"),
            rusqlite::params![rowid, json],
        )?;
        count += 1;
    }
    Ok(count)
}

/// Execute a vec0 KNN query with float32 query vector. Returns (rowid, distance) pairs.
pub fn knn_search(conn: &Connection, table: &str, query_vec: &[f32], k: usize) -> anyhow::Result<Vec<(i64, f64)>> {
    let json = format!("[{}]", query_vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    knn_search_inner(conn, table, &json, k)
}

/// Execute a vec0 KNN query with int8 query vector. Returns (rowid, distance) pairs.
pub fn knn_search_int8(conn: &Connection, table: &str, query_vec: &[i8], k: usize) -> anyhow::Result<Vec<(i64, f64)>> {
    let json = format!("[{}]", query_vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    knn_search_inner(conn, table, &json, k)
}

fn knn_search_inner(conn: &Connection, table: &str, json_vec: &str, k: usize) -> anyhow::Result<Vec<(i64, f64)>> {
    let sql = format!("SELECT rowid, distance FROM {table} WHERE embedding MATCH ?1 ORDER BY distance LIMIT ?2");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![json_vec, k as i64], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
