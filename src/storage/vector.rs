// sqlite-vec vector storage — statically compiled, no .so needed
use rusqlite::Connection;
use crate::log_warn;

/// Verify vec0 is available (statically compiled in).
pub fn try_load(conn: &Connection) -> bool {
    let ok = conn
        .execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS _vec0_test_ USING vec0(embedding FLOAT[1]);
             DROP TABLE IF EXISTS _vec0_test_;",
        )
        .is_ok();
    if !ok {
        log_warn!("vector", "vec0 not loaded: sqlite-vec extension unavailable");
    }
    ok
}

/// Create vec0 virtual tables (INT8) for this repo.
/// symbol_name_vec uses cosine distance metric; file_vec uses default (cosine).
pub fn create_tables(conn: &Connection, repo: &str, dim: usize) -> anyhow::Result<()> {
    let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    conn.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS {sym_name_table} USING vec0(embedding INT8[{dim}] distance_metric=cosine);
         CREATE VIRTUAL TABLE IF NOT EXISTS {file_table} USING vec0(embedding INT8[{dim}]);"
    ))?;
    Ok(())
}

/// Clear all vectors for a repo (drop & recreate).
pub fn clear_vectors(conn: &Connection, repo: &str, dim: usize) -> anyhow::Result<()> {
    let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    let _ = conn.execute_batch(&format!(
        "DROP TABLE IF EXISTS {sym_name_table}; DROP TABLE IF EXISTS {file_table};"
    ));
    create_tables(conn, repo, dim)
}

/// Quantize a FLOAT32 vector to INT8: round(v × 127), clamp to [-128, 127].
/// Assumes input is normalized (values roughly in [-1, 1]).
pub fn quantize_f32_to_i8(vec: &[f32]) -> Vec<i8> {
    vec.iter()
        .map(|&v| (v * 127.0).round().clamp(-128.0, 127.0) as i8)
        .collect()
}

/// Bulk insert INT8 vectors using vec_int8() wrapper.
/// Vectors are JSON i8 arrays (e.g. "[12, -45, 3, ...]").
pub fn insert_vectors_int8(
    conn: &Connection,
    table: &str,
    rows: &[(i64, &[i8])],
) -> anyhow::Result<usize> {
    let mut count = 0;
    for &(rowid, vec) in rows {
        let json = format!(
            "[{}]",
            vec.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        conn.execute(
            &format!(
                "INSERT OR IGNORE INTO {table} (rowid, embedding) VALUES (?1, vec_int8(?2))"
            ),
            rusqlite::params![rowid, json],
        )?;
        count += 1;
    }
    Ok(count)
}

/// FLOAT32 to INT8 batch insert: quantize then insert_vectors_int8.
pub fn insert_vectors_f32_as_int8(
    conn: &Connection,
    table: &str,
    rows: &[(i64, &[f32])],
) -> anyhow::Result<usize> {
    let i8_rows: Vec<(i64, Vec<i8>)> = rows
        .iter()
        .map(|(id, vec)| (*id, quantize_f32_to_i8(vec)))
        .collect();
    let slices: Vec<(i64, &[i8])> = i8_rows.iter().map(|(id, v)| (*id, v.as_slice())).collect();
    insert_vectors_int8(conn, table, &slices)
}

/// Execute a vec0 INT8 KNN query.
/// Uses vec_int8(?1) + raw BLOB for binary match.
/// Returns (rowid, distance) pairs, sorted by distance.
pub fn knn_search_int8(
    conn: &Connection,
    table: &str,
    query_vec_i8: &[i8],
    k: usize,
) -> anyhow::Result<Vec<(i64, f64)>> {
    // BLOB = raw i8 bytes (vec0 expects this for binary MATCH)
    let blob: Vec<u8> = query_vec_i8.iter().map(|&x| x as u8).collect();
    let sql = format!(
        "SELECT rowid, distance FROM {table} WHERE embedding MATCH vec_int8(?1) ORDER BY distance LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params![&blob[..], k as i64], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// FLOAT32 input → quantize → INT8 KNN. Convenience wrapper.
pub fn knn_search(
    conn: &Connection,
    table: &str,
    query_vec: &[f32],
    k: usize,
) -> anyhow::Result<Vec<(i64, f64)>> {
    let query_i8 = quantize_f32_to_i8(query_vec);
    knn_search_int8(conn, table, &query_i8, k)
}

/// Bulk insert FLOAT32 vectors (legacy — kept for testing).
pub fn insert_vectors(
    conn: &Connection,
    table: &str,
    rows: &[(i64, &[f32])],
) -> anyhow::Result<usize> {
    insert_vectors_f32_as_int8(conn, table, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    // vec0_static_init is declared in main.rs; re-declare here for test scope
    extern "C" {
        fn vec0_static_init();
    }

    static VEC0_INIT: std::sync::Once = std::sync::Once::new();

    fn with_vec0() -> rusqlite::Connection {
        VEC0_INIT.call_once(|| unsafe {
            vec0_static_init();
        });
        rusqlite::Connection::open_in_memory().unwrap()
    }

    #[test]
    fn test_int8_create_insert_match() {
        let conn = with_vec0();

        // 1. CREATE INT8 table
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS t USING vec0(embedding INT8[4] distance_metric=cosine);"
        ).expect("CREATE INT8 table");

        // 2. INSERT with vec_int8(JSON)
        conn.execute(
            "INSERT INTO t (rowid, embedding) VALUES (?1, vec_int8(?2))",
            rusqlite::params![1, "[1, -2, 3, -4]"],
        ).expect("INSERT vec_int8");
        conn.execute(
            "INSERT INTO t (rowid, embedding) VALUES (?1, vec_int8(?2))",
            rusqlite::params![2, "[2, 1, 0, -1]"],
        ).expect("INSERT vec_int8(2)");

        // 3. Verify INSERT count
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2, "Should have 2 rows");

        // 4. MATCH with vec_int8(?1) + BLOB → binary match
        let query: Vec<i8> = vec![1, -1, 2, -3];
        let blob: Vec<u8> = query.into_iter().map(|x| x as u8).collect();

        let mut stmt = conn
            .prepare(
                "SELECT rowid, distance FROM t WHERE embedding MATCH vec_int8(?1) ORDER BY distance LIMIT 3",
            )
            .expect("prepare MATCH");
        let rows = stmt
            .query_map(rusqlite::params![&blob[..]], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
            })
            .expect("execute MATCH");

        let results: Vec<(i64, f64)> = rows.filter_map(|r| r.ok()).collect();
        assert!(!results.is_empty(), "MATCH should return results");
        assert_eq!(
            results[0].0, 1,
            "rowid 1 should be closest (identical vector)"
        );
        assert!(results[0].1 < results[1].1, "distance should increase");
    }

    #[test]
    fn test_quantize_round_trip() {
        let original = vec![-1.0, 0.0, 0.5, 1.0];
        let quantized = quantize_f32_to_i8(&original);
        assert_eq!(quantized, vec![-127, 0, 64, 127]);
    }

    #[test]
    fn test_knn_search_int8_integration() {
        let conn = with_vec0();

        // Create an INT8 table via create_tables
        create_tables(&conn, "test_repo", 4).unwrap();

        let table = "symbol_name_vec_test_repo";

        // Insert vectors via insert_vectors_int8
        let v1: Vec<i8> = vec![10, 20, 30, 40];
        let v2: Vec<i8> = vec![-10, -20, -30, -40];
        let rows = vec![(1i64, v1.as_slice()), (2i64, v2.as_slice())];
        let count = insert_vectors_int8(&conn, table, &rows).unwrap();
        assert_eq!(count, 2);

        // Query with a vector close to v1
        let query = vec![10i8, 20, 30, 39]; // slightly different from v1
        let results = knn_search_int8(&conn, table, &query, 3).unwrap();
        assert!(!results.is_empty(), "knn should return results");
        assert_eq!(results[0].0, 1, "rowid 1 should be closest");
    }
}
