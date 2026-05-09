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
             JOIN symbols s1 ON e.source_id = s1.id \
             LEFT JOIN symbols s2 ON e.target_id = s2.id \
             WHERE e.source_id = ?1 AND e.target_id != 0 {}",
            filter_clause
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let rows = stmt.query_map(rusqlite::params![symbol_id], |row| {
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
             JOIN symbols s2 ON e.source_id = s2.id \
             LEFT JOIN symbols s1 ON e.target_id = s1.id \
             WHERE e.target_id = ?1 AND e.source_id != 0 {}",
            filter_clause
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            let rows = stmt.query_map(rusqlite::params![symbol_id], |row| {
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
    conn.query_row(
        "SELECT s.id FROM symbols s JOIN branches b ON s.id = b.symbol_id \
         WHERE s.name = ?1 AND b.repo = ?2 AND (b.branch_name = ?3 OR b.branch_name IS NULL)",
        rusqlite::params![name, repo, branch],
        |row| row.get(0),
    ).ok()
}

/// Get symbol name by id.
pub fn symbol_name_by_id(conn: &Connection, id: i64) -> Option<String> {
    conn.query_row("SELECT name FROM symbols WHERE id = ?1", rusqlite::params![id], |r| r.get(0)).ok()
}

/// Build forward adjacency map: source_id → [(target_id, edge_type)]
pub fn build_forward_adj(
    conn: &Connection,
    edge_filter: &[String],
) -> HashMap<i64, Vec<(i64, String)>> {
    let filter = if edge_filter.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = edge_filter.iter().map(|p| format!("edge_type LIKE '{}%'", p.replace('\'', "''"))).collect();
        format!("WHERE {}", parts.join(" OR "))
    };
    let sql = format!(
        "SELECT source_id, target_id, edge_type FROM edges WHERE target_id != 0 {}",
        filter
    );
    let mut adj: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
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
) -> HashMap<i64, Vec<(i64, String)>> {
    let filter = if edge_filter.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = edge_filter.iter().map(|p| format!("edge_type LIKE '{}%'", p.replace('\'', "''"))).collect();
        format!("WHERE {}", parts.join(" OR "))
    };
    let sql = format!(
        "SELECT target_id, source_id, edge_type FROM edges WHERE source_id != 0 {}",
        filter
    );
    let mut adj: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map([], |row| {
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
    mode: &str,         // "shortest" or "all"
    max_paths: usize,
) -> Vec<PathResult> {
    let fwd_adj = build_forward_adj(conn, edge_filter);
    let rev_adj = build_reverse_adj(conn, edge_filter);

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
) -> Vec<ImpactResult> {
    let fwd_adj = build_forward_adj(conn, edge_filter);
    let rev_adj = build_reverse_adj(conn, edge_filter);
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
    depth: usize,
    edge_filter: &[String],
) -> NeighborMap {
    let mut map = NeighborMap {
        forward: HashMap::new(),
        backward: HashMap::new(),
    };

    let edges = get_edges(conn, symbol_id, direction, edge_filter);

    for e in &edges {
        // Determine if this edge is forward or backward relative to the queried symbol
        let prefix = edge_prefix(&e.edge_type);
        if e.source_id == symbol_id {
            // Forward: we are the source
            map.forward.entry(prefix.to_string())
                .or_default()
                .push(e.target_name.clone());
        } else {
            // Backward: we are the target
            map.backward.entry(prefix.to_string())
                .or_default()
                .push(e.source_name.clone());
        }
    }

    // If depth > 1, recurse for each neighbor
    if depth > 1 {
        let neighbor_ids: Vec<(i64, bool)> = edges.iter().map(|e| {
            if e.source_id == symbol_id {
                (e.target_id, true)  // forward neighbor
            } else {
                (e.source_id, false) // backward neighbor
            }
        }).collect();

        for (nid, is_fwd) in &neighbor_ids {
            let sub_edges = get_edges(conn, *nid, "both", edge_filter);
            for se in &sub_edges {
                if se.source_id == *nid {
                    let label = format!("depth2:{}", edge_prefix(&se.edge_type));
                    if *is_fwd {
                        map.forward.entry(label).or_default().push(se.target_name.clone());
                    } else {
                        map.backward.entry(label).or_default().push(se.target_name.clone());
                    }
                }
            }
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
        "SELECT s.name FROM edges e JOIN symbols s ON e.target_id = s.id WHERE e.source_id = ?1 AND e.edge_type LIKE 'uses:%'"
    ).and_then(|mut s| {
        s.query_map(rusqlite::params![symbol_id], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
    }).unwrap_or_default();

    let refs: Vec<String> = conn.prepare(
        "SELECT s.name FROM edges e JOIN symbols s ON e.target_id = s.id WHERE e.source_id = ?1 AND e.edge_type LIKE 'references:%'"
    ).and_then(|mut s| {
        s.query_map(rusqlite::params![symbol_id], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
    }).unwrap_or_default();

    let literals: Vec<String> = conn.prepare(
        "SELECT s.name FROM edges e JOIN symbols s ON e.target_id = s.id WHERE e.source_id = ?1 AND s.kind = 'string_literal'"
    ).and_then(|mut s| {
        s.query_map(rusqlite::params![symbol_id], |r| r.get::<_, String>(0))
            .map(|rows| rows.flatten().collect())
    }).unwrap_or_default();

    (uses, refs, literals)
}
