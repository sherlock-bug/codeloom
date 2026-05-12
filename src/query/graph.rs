// Shared graph traversal engine for MCP analysis tools.
// Pure SQL → no graph DB dependency.

use rusqlite::Connection;
use std::collections::{HashMap, HashSet, VecDeque};

/// An edge from one symbol to another.
pub struct Edge {
    pub source_id: i64,
    pub target_id: i64,
    pub edge_type: String,
    pub source_name: String,
    pub target_name: String,
}

/// Neighbors grouped by direction and edge type prefix.
pub struct NeighborMap {
    pub forward: HashMap<String, Vec<String>>,   // edge_prefix → [target_names]
    pub backward: HashMap<String, Vec<String>>,  // edge_prefix → [source_names]
}

/// Fetch all edges for a symbol, filtered by direction and optional edge types.
pub fn get_edges(
    conn: &Connection,
    symbol_id: i64,
    direction: &str,
    edge_filter: &[String],
    branch_id: i64,
) -> Vec<Edge> {
    let mut edges = Vec::new();
    let filter_clause = if edge_filter.is_empty() {
        String::new()
    } else {
        let prefixes: Vec<String> = edge_filter.iter().map(|p| format!("e.edge_type LIKE '{}%'", p.replace('\'', "''"))).collect();
        format!("AND ({})", prefixes.join(" OR "))
    };

    if direction == "forward" || direction == "both" {
        let sql = format!(
            "SELECT e.source_id, e.target_id, e.edge_type, s1.name, s2.name \
             FROM edges e \
             JOIN nodes s1 ON e.source_id = s1.id \
             LEFT JOIN nodes s2 ON e.target_id = s2.id \
             WHERE e.source_id = ?1 AND e.target_id != 0 AND (e.branch_id = ?2 OR e.branch_id = 0) {}",
            filter_clause
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let rows = stmt.query_map(rusqlite::params![symbol_id, branch_id], |row| {
                Ok(Edge {
                    source_id: row.get(0)?,
                    target_id: row.get(1)?,
                    edge_type: row.get(2)?,
                    source_name: row.get(3)?,
                    target_name: row.get::<_, String>(4).unwrap_or_default(),
                })
            });
            if let Ok(rows) = rows {
                for r in rows.flatten() { edges.push(r); }
            }
        }
    }

    if direction == "reverse" || direction == "both" {
        let sql = format!(
            "SELECT e.source_id, e.target_id, e.edge_type, s1.name, s2.name \
             FROM edges e \
             JOIN nodes s2 ON e.source_id = s2.id \
             LEFT JOIN nodes s1 ON e.target_id = s1.id \
             WHERE e.target_id = ?1 AND e.source_id != 0 AND (e.branch_id = ?2 OR e.branch_id = 0) {}",
            filter_clause
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let rows = stmt.query_map(rusqlite::params![symbol_id, branch_id], |row| {
                Ok(Edge {
                    source_id: row.get(0)?,
                    target_id: row.get(1)?,
                    edge_type: row.get(2)?,
                    source_name: row.get(3)?,
                    target_name: row.get::<_, String>(4).unwrap_or_default(),
                })
            });
            if let Ok(rows) = rows {
                for r in rows.flatten() { edges.push(r); }
            }
        }
    }

    edges
}

/// Resolve a symbol name to its internal ID.
pub fn resolve_symbol_id(conn: &Connection, name: &str, repo: &str, branch: &str) -> Option<i64> {
    let branch_id = crate::storage::resolve_branch_id(conn, repo, branch).ok()?;
    conn.query_row(
        "SELECT n.id FROM nodes n JOIN branches b ON n.id = b.node_id \
         WHERE n.name = ?1 AND n.node_type='sym' AND b.repo = ?2 AND (b.branch_id = ?3 OR b.branch_id = 0)",
        rusqlite::params![name, repo, branch_id],
        |row| row.get(0),
    ).ok()
}

/// Resolve symbol by id (preferred) or name (fallback).
/// Returns None if neither resolves.
pub fn resolve_symbol_id_or_name(
    conn: &Connection,
    id_opt: Option<i64>,
    name: &str,
    repo: &str,
    branch: &str,
) -> Option<i64> {
    if let Some(sid) = id_opt {
        // Validate the id exists
        conn.query_row(
            "SELECT 1 FROM nodes WHERE id=?1 AND node_type='sym'",
            rusqlite::params![sid],
            |_| Ok(()),
        )
        .ok()
        .map(|_| sid)
    } else if !name.is_empty() {
        resolve_symbol_id(conn, name, repo, branch)
    } else {
        None
    }
}

/// Get symbol name by id.
pub fn symbol_name_by_id(conn: &Connection, id: i64) -> Option<String> {
    conn.query_row("SELECT name FROM nodes WHERE id = ?1", rusqlite::params![id], |r| r.get(0)).ok()
}

/// Build forward adjacency map: source_id → [(target_id, edge_type)]
/// branch_id: filter edges by branch (0 = unassigned, backward compatible).
pub fn build_forward_adj(
    conn: &Connection,
    edge_filter: &[String],
    branch_id: i64,
) -> HashMap<i64, Vec<(i64, String)>> {
    let filter = if edge_filter.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = edge_filter.iter().map(|p| format!("edge_type LIKE '{}%'", p.replace('\'', "''"))).collect();
        format!("AND {}", parts.join(" OR "))
    };
    let sql = format!(
        "SELECT source_id, target_id, edge_type FROM edges WHERE target_id != 0 AND (branch_id = ?1 OR branch_id = 0) {}",
        filter
    );
    let mut adj: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![branch_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?))
        }) {
            for r in rows.flatten() {
                adj.entry(r.0).or_default().push((r.1, r.2));
            }
        }
    }
    adj
}

/// Build reverse adjacency map: target_id → [(source_id, edge_type)]
pub fn build_reverse_adj(
    conn: &Connection,
    edge_filter: &[String],
    branch_id: i64,
) -> HashMap<i64, Vec<(i64, String)>> {
    let filter = if edge_filter.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = edge_filter.iter().map(|p| format!("edge_type LIKE '{}%'", p.replace('\'', "''"))).collect();
        format!("AND {}", parts.join(" OR "))
    };
    let sql = format!(
        "SELECT target_id, source_id, edge_type FROM edges WHERE source_id != 0 AND (branch_id = ?1 OR branch_id = 0) {}",
        filter
    );
    let mut adj: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![branch_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?))
        }) {
            for r in rows.flatten() {
                adj.entry(r.0).or_default().push((r.1, r.2));
            }
        }
    }
    adj
}

/// BFS path search between source and target.
/// Returns either the shortest path or all paths up to max_paths.
pub struct PathResult {
    pub edges: Vec<(String, String, String)>, // (from_name, edge_type, to_name)
}

pub fn bfs_path_search(
    conn: &Connection,
    source_id: i64,
    target_id: i64,
    edge_filter: &[String],
    max_depth: usize,
    mode: &str,
    max_paths: usize,
    branch_id: i64,
) -> Vec<PathResult> {
    let fwd_adj = build_forward_adj(conn, edge_filter, branch_id);
    let rev_adj = build_reverse_adj(conn, edge_filter, branch_id);

    let mut results: Vec<PathResult> = Vec::new();
    let mut visited_nodes: HashSet<i64> = HashSet::new();

    // Simple BFS — each queue entry: (node_id, path_of_node_ids, path_of_edges)
    let mut queue: VecDeque<(i64, Vec<i64>, Vec<(i64, String, i64)>)> = VecDeque::new();
    queue.push_back((source_id, vec![source_id], vec![]));

    while let Some((current, node_path, edge_path)) = queue.pop_front() {
        if node_path.len() > max_depth + 1 { continue; }

        if current == target_id {
            let name_map = |id: i64| symbol_name_by_id(conn, id).unwrap_or_else(|| format!("id_{}", id));
            let edges: Vec<(String, String, String)> = edge_path.iter().map(|(from, etype, to)| {
                (name_map(*from), etype.clone(), name_map(*to))
            }).collect();
            results.push(PathResult { edges });
            if mode == "shortest" || results.len() >= max_paths { break; }
            continue;
        }

        // Forward
        if let Some(neighbors) = fwd_adj.get(&current) {
            for (next_id, etype) in neighbors {
                if node_path.contains(next_id) { continue; } // cycle detection
                let mut np = node_path.clone();
                np.push(*next_id);
                let mut ep = edge_path.clone();
                ep.push((current, etype.clone(), *next_id));
                queue.push_back((*next_id, np, ep));
            }
        }

        // Reverse
        if let Some(neighbors) = rev_adj.get(&current) {
            for (prev_id, etype) in neighbors {
                if node_path.contains(prev_id) { continue; } // cycle detection
                // Reverse edge semantics: prev_id --(etype)--> current
                // We walk from current backward to prev_id
                let mut np = node_path.clone();
                np.push(*prev_id);
                let mut ep = edge_path.clone();
                ep.push((current, format!("←{}", etype), *prev_id));
                queue.push_back((*prev_id, np, ep));
            }
        }
    }

    results
}

/// Transitive closure from a symbol, limited by radius.
pub struct ImpactResult {
    pub symbol: String,
    pub distance: usize,
    pub via: String,
}

pub fn transitive_closure(
    conn: &Connection,
    symbol_id: i64,
    direction: &str,
    radius: usize,
    edge_filter: &[String],
    branch_id: i64,
) -> Vec<ImpactResult> {
    let fwd_adj = build_forward_adj(conn, edge_filter, branch_id);
    let rev_adj = build_reverse_adj(conn, edge_filter, branch_id);
    let mut results: Vec<ImpactResult> = Vec::new();
    let mut visited: HashSet<i64> = HashSet::new();
    visited.insert(symbol_id);

    // BFS with distance tracking: (node_id, distance, via_edge_type)
    let mut queue: VecDeque<(i64, usize, String)> = VecDeque::new();

    // Seed with direct neighbors
    if direction == "forward" || direction == "both" {
        if let Some(neighbors) = fwd_adj.get(&symbol_id) {
            for (next_id, etype) in neighbors {
                if !visited.contains(next_id) {
                    visited.insert(*next_id);
                    let name = symbol_name_by_id(conn, *next_id).unwrap_or_default();
                    results.push(ImpactResult { symbol: name, distance: 1, via: etype.clone() });
                    queue.push_back((*next_id, 1, etype.clone()));
                }
            }
        }
    }
    if direction == "reverse" || direction == "both" {
        if let Some(neighbors) = rev_adj.get(&symbol_id) {
            for (prev_id, etype) in neighbors {
                if !visited.contains(prev_id) {
                    visited.insert(*prev_id);
                    let name = symbol_name_by_id(conn, *prev_id).unwrap_or_default();
                    results.push(ImpactResult { symbol: name, distance: 1, via: format!("←{}", etype) });
                    queue.push_back((*prev_id, 1, etype.clone()));
                }
            }
        }
    }

    while let Some((current, dist, _)) = queue.pop_front() {
        if dist >= radius { continue; }
        let next_dist = dist + 1;

        if direction == "forward" || direction == "both" {
            if let Some(neighbors) = fwd_adj.get(&current) {
                for (next_id, etype) in neighbors {
                    if !visited.contains(next_id) {
                        visited.insert(*next_id);
                        let name = symbol_name_by_id(conn, *next_id).unwrap_or_default();
                        results.push(ImpactResult { symbol: name, distance: next_dist, via: etype.clone() });
                        queue.push_back((*next_id, next_dist, etype.clone()));
                    }
                }
            }
        }
        if direction == "reverse" || direction == "both" {
            if let Some(neighbors) = rev_adj.get(&current) {
                for (prev_id, etype) in neighbors {
                    if !visited.contains(prev_id) {
                        visited.insert(*prev_id);
                        let name = symbol_name_by_id(conn, *prev_id).unwrap_or_default();
                        results.push(ImpactResult { symbol: name, distance: next_dist, via: format!("←{}", etype) });
                        queue.push_back((*prev_id, next_dist, etype.clone()));
                    }
                }
            }
        }
    }

    // Sort by distance
    results.sort_by_key(|r| r.distance);
    results
}

/// Build neighbor map for a single symbol.
pub fn neighbor_map(
    conn: &Connection,
    symbol_id: i64,
    direction: &str,
    edge_filter: &[String],
    branch_id: i64,
) -> NeighborMap {
    let mut map = NeighborMap {
        forward: HashMap::new(),
        backward: HashMap::new(),
    };

    // Check if this symbol is a class/struct — skip members, expose their refs
    let is_class_or_struct = conn
        .query_row(
            "SELECT 1 FROM nodes WHERE id=?1 AND kind IN ('class','struct')",
            rusqlite::params![symbol_id],
            |_| Ok(()),
        )
        .is_ok();

    // Get member ids for class/struct (via contains: edges)
    let member_ids: HashSet<i64> = if is_class_or_struct {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT e.target_id FROM edges e WHERE e.source_id=?1 AND e.edge_type='contains' AND (e.branch_id=?2 OR e.branch_id=0)"
        ) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![symbol_id, branch_id], |r| r.get::<_, i64>(0)) {
                rows.flatten().collect()
            } else {
                HashSet::new()
            }
        } else {
            HashSet::new()
        }
    } else {
        HashSet::new()
    };

    let edges = get_edges(conn, symbol_id, direction, edge_filter, branch_id);

    let mut expanded = HashSet::new();

    for e in &edges {
        let prefix = edge_prefix(&e.edge_type);

        // For class/struct: skip contains: edges to members, expose member refs instead
        if is_class_or_struct && e.source_id == symbol_id && member_ids.contains(&e.target_id) {
            if expanded.insert(e.target_id) {
                for me in get_edges(conn, e.target_id, "forward", &[], branch_id) {
                    let mp = format!("via_member:{}", edge_prefix(&me.edge_type));
                    map.forward
                        .entry(mp)
                        .or_default()
                        .push(me.target_name.clone());
                }
            }
            continue;
        }

        if e.source_id == symbol_id {
            map.forward
                .entry(prefix.to_string())
                .or_default()
                .push(e.target_name.clone());
        } else {
            map.backward
                .entry(prefix.to_string())
                .or_default()
                .push(e.source_name.clone());
        }
    }

    map
}

/// Extract edge type prefix (e.g. "calls:kill(2)" → "calls").
pub fn edge_prefix(edge_type: &str) -> &str {
    edge_type.split(':').next().unwrap_or(edge_type)
}

/// Get terminal symbol dependencies: uses, references, string literals
pub fn get_terminal_deps(conn: &Connection, symbol_id: i64) -> (Vec<String>, Vec<String>, Vec<String>) {
    let uses: Vec<String> = conn.prepare(
        "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id WHERE e.source_id = ?1 AND e.edge_type LIKE 'uses:%'"
    ).and_then(|mut s| {
        s.query_map(rusqlite::params![symbol_id], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
    }).unwrap_or_default();

    let refs: Vec<String> = conn.prepare(
        "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id WHERE e.source_id = ?1 AND e.edge_type LIKE 'references:%'"
    ).and_then(|mut s| {
        s.query_map(rusqlite::params![symbol_id], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
    }).unwrap_or_default();

    let literals: Vec<String> = conn.prepare(
        "SELECT n.name FROM edges e JOIN nodes n ON e.target_id = n.id WHERE e.source_id = ?1 AND n.kind = 'string_literal'"
    ).and_then(|mut s| {
        s.query_map(rusqlite::params![symbol_id], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
    }).unwrap_or_default();

    (uses, refs, literals)
}
