// Call graph traversal — type-aware call resolution with recursive depth control
use rusqlite::Connection;
use std::collections::HashSet;

/// Get the call graph for a symbol, traversing callers or callees up to max_depth.
/// Returns a formatted text representation of the call tree.
pub fn get_call_graph(
    conn: &Connection,
    name: &str,
    repo: &str,
    branch: &str,
    direction: &str,
    max_depth: usize,
) -> String {
    let bwc = format!("AND (b.branch_name = '{}' OR b.branch_name IS NULL)", branch.replace('\'', "''"));
    let exact_sql = format!(
        "SELECT s.id FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name=?2 {}",
        bwc
    );
    let sym_ids: Vec<i64> = match conn.prepare(&exact_sql) {
        Ok(mut stmt) => stmt
            .query_map(rusqlite::params![repo, name], |r| r.get(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default(),
        Err(_) => return format!("Error querying symbol '{}'", name),
    };
    if sym_ids.is_empty() {
        let like = format!("%{}%", name);
        let like_sql = format!(
            "SELECT s.id, s.name FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name LIKE ?2 AND s.kind NOT IN ('string_literal', 'enum_value') {} LIMIT 10",
            bwc
        );
        let similar: Vec<(i64, String)> = match conn.prepare(&like_sql) {
            Ok(mut stmt) => stmt
                .query_map(rusqlite::params![repo, like], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default(),
            Err(_) => vec![],
        };
        if similar.is_empty() {
            return format!(
                "Symbol '{}' not found in {} (branch={})",
                name, repo, branch
            );
        }
        let (first_id, ref first_name) = similar[0];
        let mut out = format!(
            "Call graph for '{}' -> auto-matched '{}' ({}):\n",
            name, first_name, direction
        );
        let mut visited = HashSet::new();
        visited.insert(first_id);
        out.push_str(&format!("  * {} (id={})\n", first_name, first_id));
        traverse_calls(branch, conn, first_id, direction, max_depth, 1, &mut visited, &mut out);
        return out;
    }
    let mut out = format!("Call graph for '{}' ({}):\n", name, direction);
    let mut visited = HashSet::new();
    for &root_id in &sym_ids {
        visited.insert(root_id);
        out.push_str(&format!("  * {} (id={})\n", name, root_id));
        traverse_calls(branch, conn, root_id, direction, max_depth, 1, &mut visited, &mut out);
    }
    out
}

fn traverse_calls(
    branch: &str,
    conn: &Connection,
    sym_id: i64,
    direction: &str,
    max_depth: usize,
    depth: usize,
    visited: &mut HashSet<i64>,
    out: &mut String,
) {
    if depth > max_depth {
        return;
    }
    let prefix = "  ".repeat(depth + 1);
    let query = match direction {
        "callees" => format!(
            "SELECT e.target_id, e.edge_type FROM edges e WHERE e.source_id={} AND e.target_id!=0 AND (e.edge_type LIKE 'calls:%' OR e.edge_type LIKE 'calls_override:%')",
            sym_id
        ),
        _ => format!(
            "SELECT e.source_id, e.edge_type FROM edges e WHERE e.target_id={} AND (e.edge_type LIKE 'calls:%' OR e.edge_type LIKE 'calls_override:%')",
            sym_id
        ),
    };
    if let Ok(mut stmt) = conn.prepare(&query) {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                let (other_id, edge_type) = row;
                if visited.contains(&other_id) {
                    let repeated_name = conn
                        .query_row(
                            "SELECT name FROM symbols WHERE id=?1",
                            rusqlite::params![other_id],
                            |r| r.get::<_, String>(0),
                        )
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "{}{} {} (already shown)\n",
                        prefix, '→', repeated_name
                    ));
                    continue;
                }
                visited.insert(other_id);
                let other_name = conn
                    .query_row(
                        "SELECT s.name FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.id=?1 AND (b.branch_name=?2 OR b.branch_name IS NULL)",
                        rusqlite::params![other_id, branch],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_default();
                let called = edge_type.strip_prefix("calls:").unwrap_or(&edge_type);
                out.push_str(&format!(
                    "{}{} {} (calls:{})\n",
                    prefix, '→', other_name, called
                ));
                if depth < max_depth {
                    traverse_calls(
                        branch, conn, other_id, direction, max_depth, depth + 1, visited, out,
                    );
                }
            }
        }
    }
}
