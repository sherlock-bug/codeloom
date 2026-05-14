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
    sym_id: Option<i64>,
) -> String {
    let branch_id = crate::storage::resolve_branch_id(conn, repo, branch).unwrap_or(0);
    let bwc = format!("AND (b.branch_id = {} OR b.branch_id = 0)", branch_id);
    let sym_ids: Vec<i64> = if let Some(sid) = sym_id {
        vec![sid]
    } else {
        let exact_sql = format!(
            "SELECT n.id FROM nodes n JOIN branches b ON n.id=b.node_id WHERE n.repo=?1 AND n.name=?2 AND n.node_type='sym' {}",
            bwc
        );
        match conn.prepare(&exact_sql) {
            Ok(mut stmt) => stmt
                .query_map(rusqlite::params![repo, name], |r| r.get(0))
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default(),
            Err(_) => return format!("Error querying symbol '{}'", name),
        }
    };
    if sym_ids.is_empty() {
        let like = format!("%{}%", name);
        let like_sql = format!(
            "SELECT n.id, n.name FROM nodes n JOIN branches b ON n.id=b.node_id WHERE n.repo=?1 AND n.name LIKE ?2 AND n.node_type='sym' AND n.kind NOT IN ('string_literal', 'enum_value') {} LIMIT 10",
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
        traverse_calls(branch_id, conn, first_id, direction, max_depth, 1, &mut visited, &mut out);
        return out;
    }
    let mut out = format!("Call graph for '{}' ({}):\n", name, direction);
    let mut visited = HashSet::new();
    for &root_id in &sym_ids {
        visited.insert(root_id);
        out.push_str(&format!("  * {} (id={})\n", name, root_id));
        traverse_calls(branch_id, conn, root_id, direction, max_depth, 1, &mut visited, &mut out);
    }
    out
}

fn traverse_calls(
    branch_id: i64,
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
            "SELECT e.target_id, e.edge_type FROM edges e WHERE e.source_id={} AND e.target_id!=0 AND e.branch_id=? AND (e.edge_type LIKE 'uses:%' OR e.edge_type LIKE 'calls:%' OR e.edge_type LIKE 'calls_override:%')",
            sym_id
        ),
        _ => format!(
            "SELECT e.source_id, e.edge_type FROM edges e WHERE e.target_id={} AND e.branch_id=? AND (e.edge_type LIKE 'uses:%' OR e.edge_type LIKE 'calls:%' OR e.edge_type LIKE 'calls_override:%')",
            sym_id
        ),
    };
    if let Ok(mut stmt) = conn.prepare(&query) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![branch_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        }) {
            for row in rows.flatten() {
                let (other_id, edge_type) = row;
                if visited.contains(&other_id) {
                    let repeated_name = conn
                        .query_row(
                            "SELECT name FROM nodes WHERE id=?1",
                            rusqlite::params![other_id],
                            |r| r.get::<_, String>(0),
                        )
                        .unwrap_or_default();
                    // Node already shown — but if this is a different edge type (e.g.
                    // uses: vs calls:), append it as an alias annotation
                    let label = edge_type.split(':').next().unwrap_or("edge");
                    let prefix_stripped = edge_type.find(':').map(|p| &edge_type[p+1..]).unwrap_or(&edge_type);
                    if label != "calls" && label != "calls_override" {
                        out.push_str(&format!(
                            "{}   also ({}) ({})\n",
                            prefix, label, prefix_stripped
                        ));
                    } else {
                        out.push_str(&format!(
                            "{}{} {} (already shown)\n",
                            prefix, '→', repeated_name
                        ));
                    }
                    continue;
                }
                visited.insert(other_id);
                let other_name = conn
                    .query_row(
                        "SELECT n.name FROM nodes n LEFT JOIN branches b ON n.id=b.node_id WHERE n.id=?1 AND n.node_type='sym' AND (b.branch_id=?2 OR b.branch_id=0 OR b.branch_id IS NULL) LIMIT 1",
                        rusqlite::params![other_id, branch_id],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap_or_default();
                let prefix_stripped = edge_type.find(':').map(|p| &edge_type[p+1..]).unwrap_or(&edge_type);
                out.push_str(&format!(
                    "{}{} {} ({}:{})\n",
                    prefix, '→', other_name, edge_type.split(':').next().unwrap_or("edge"), prefix_stripped
                ));
                if depth < max_depth {
                    traverse_calls(
                        branch_id, conn, other_id, direction, max_depth, depth + 1, visited, out,
                    );
                } else {
                    // Terminal node — show terminal dependencies
                    let (uses, refs, literals) = crate::query::graph::get_terminal_deps(conn, other_id);
                    if !uses.is_empty() {
                        out.push_str(&format!("{}   uses: {}\n", prefix, uses.join(", ")));
                    }
                    if !refs.is_empty() {
                        out.push_str(&format!("{}   references: {}\n", prefix, refs.join(", ")));
                    }
                    if !literals.is_empty() {
                        out.push_str(&format!("{}   string_literals: {}\n", prefix, literals.join(", ")));
                    }
                }
            }
        }
    }
}
