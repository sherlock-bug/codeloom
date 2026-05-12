use rusqlite::Connection;
use crate::log_info;

/// Initialize DB schema for clean-slate indexing (v0.9+).
/// No legacy tables — only nodes/edges/branches/fts5_all/branch_meta.
pub fn run(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS branch_meta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo TEXT NOT NULL,
            branch_name TEXT NOT NULL DEFAULT 'main',
            UNIQUE(repo, branch_name)
        );

        CREATE TABLE IF NOT EXISTS nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo TEXT NOT NULL,
            node_type TEXT NOT NULL,
            name TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            file_path TEXT NOT NULL DEFAULT '',
            line_start INTEGER DEFAULT 0,
            content_hash TEXT DEFAULT '',
            branch_id INTEGER NOT NULL DEFAULT 0,
            kind TEXT DEFAULT '',
            attrs TEXT DEFAULT '{}',
            UNIQUE(content_hash, file_path, name, branch_id, repo)
        );
        CREATE INDEX IF NOT EXISTS idx_nodes_repo ON nodes(repo);
        CREATE INDEX IF NOT EXISTS idx_nodes_type ON nodes(node_type);
        CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
        CREATE INDEX IF NOT EXISTS idx_nodes_hash ON nodes(content_hash);
        CREATE INDEX IF NOT EXISTS idx_nodes_name ON nodes(name);

        CREATE TABLE IF NOT EXISTS edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL,
            target_id INTEGER NOT NULL,
            edge_type TEXT NOT NULL,
            source_repo TEXT, target_repo TEXT,
            branch_id INTEGER NOT NULL DEFAULT 0
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_ed_unique ON edges(source_id, target_id, edge_type, branch_id);
        CREATE INDEX IF NOT EXISTS idx_ed_tgt ON edges(target_id);
        CREATE INDEX IF NOT EXISTS idx_ed_type ON edges(edge_type);

        CREATE TABLE IF NOT EXISTS branches (
            node_id INTEGER NOT NULL,
            repo TEXT NOT NULL DEFAULT 'default',
            branch_id INTEGER NOT NULL,
            branch_name TEXT NOT NULL DEFAULT '',
            override_def TEXT, override_hash TEXT,
            PRIMARY KEY (node_id, repo, branch_id)
        );

        CREATE TABLE IF NOT EXISTS git_index_state (
            repo TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            head_commit TEXT NOT NULL,
            parent_ref TEXT,
            ref_type TEXT DEFAULT 'branch',
            indexed_files INTEGER DEFAULT 0,
            indexed_at TEXT NOT NULL,
            PRIMARY KEY (repo, branch_name)
        );

        CREATE TABLE IF NOT EXISTS branch_glossary (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            alias TEXT NOT NULL,
            description TEXT, doc_path TEXT,
            UNIQUE(repo, branch_name, alias)
        );

        CREATE TABLE IF NOT EXISTS doc_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            doc_node_id INTEGER NOT NULL,
            alt_text TEXT,
            original_src TEXT,
            image_data BLOB,
            position INTEGER,
            section_context TEXT,
            image_type TEXT DEFAULT 'inline',
            original_size INTEGER,
            compressed_size INTEGER,
            width INTEGER,
            height INTEGER
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS fts5_all USING fts5(name, content);
    ")?;
    log_info!("storage::schema", "schema run: all tables created/verified");
    Ok(())
}
