use rusqlite::Connection;

pub fn run(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY AUTOINCREMENT, repo TEXT NOT NULL DEFAULT 'default',
            name TEXT NOT NULL, kind TEXT NOT NULL, definition TEXT NOT NULL,
            content_hash TEXT NOT NULL, file_path TEXT NOT NULL,
            line_start INTEGER NOT NULL DEFAULT 0, line_end INTEGER NOT NULL DEFAULT 0,
            language TEXT, signature TEXT, parent_class TEXT, namespace TEXT,
            UNIQUE(content_hash, file_path, name, repo)
        );
        CREATE INDEX IF NOT EXISTS idx_sym_name ON symbols(name);
        CREATE INDEX IF NOT EXISTS idx_sym_kind ON symbols(kind);
        CREATE INDEX IF NOT EXISTS idx_sym_repo ON symbols(repo);
        CREATE INDEX IF NOT EXISTS idx_sym_hash ON symbols(content_hash);

        CREATE TABLE IF NOT EXISTS edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL, target_id INTEGER NOT NULL,
            edge_type TEXT NOT NULL, source_repo TEXT, target_repo TEXT, branch_name TEXT
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_ed_unique ON edges(source_id, edge_type, source_repo);
        CREATE INDEX IF NOT EXISTS idx_ed_tgt ON edges(target_id);
        CREATE INDEX IF NOT EXISTS idx_ed_type ON edges(edge_type);

        CREATE TABLE IF NOT EXISTS branches (
            symbol_id INTEGER NOT NULL, repo TEXT NOT NULL DEFAULT 'default',
            branch_name TEXT NOT NULL, override_def TEXT, override_hash TEXT,
            PRIMARY KEY (symbol_id, repo, branch_name)
        );

        CREATE TABLE IF NOT EXISTS git_index_state (
            repo TEXT NOT NULL, branch_name TEXT NOT NULL, head_commit TEXT NOT NULL,
            parent_ref TEXT, ref_type TEXT DEFAULT 'branch',
            indexed_files INTEGER DEFAULT 0, indexed_at TEXT NOT NULL,
            PRIMARY KEY (repo, branch_name)
        );

        CREATE TABLE IF NOT EXISTS branch_glossary (
            id INTEGER PRIMARY KEY AUTOINCREMENT, repo TEXT NOT NULL,
            branch_name TEXT NOT NULL, alias TEXT NOT NULL, description TEXT, doc_path TEXT,
            UNIQUE(repo, branch_name, alias)
        );

        CREATE TABLE IF NOT EXISTS doc_nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT, repo TEXT DEFAULT 'default',
            title TEXT, section_path TEXT, content TEXT, level INTEGER,
            file_path TEXT NOT NULL, file_format TEXT, branch_name TEXT
        );

        CREATE TABLE IF NOT EXISTS doc_code_links (
            doc_node_id INTEGER REFERENCES doc_nodes(id),
            symbol_id INTEGER REFERENCES symbols(id),
            link_type TEXT, strength REAL DEFAULT 0.0, source TEXT DEFAULT 'embedding',
            PRIMARY KEY (doc_node_id, symbol_id)
        );
    ")?;
    Ok(())
}
