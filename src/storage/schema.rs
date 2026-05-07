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
            id INTEGER PRIMARY KEY, repo TEXT DEFAULT 'default',
            title TEXT, section_path TEXT, content TEXT, level INTEGER,
            file_path TEXT NOT NULL, file_format TEXT, branch_name TEXT,
            content_hash TEXT,
            UNIQUE(repo, file_path, section_path)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS fts5_sym USING fts5(name, file_path, signature);
        CREATE VIRTUAL TABLE IF NOT EXISTS fts5_doc USING fts5(title, section_path, content);
    ")?;
    // v0.5.0 migrations: multi-format doc + image support
    migrate_v5(conn)?;
    Ok(())
}

fn migrate_v5(conn: &Connection) -> anyhow::Result<()> {
    // doc_images table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS doc_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            doc_node_id INTEGER NOT NULL REFERENCES doc_nodes(id) ON DELETE CASCADE,
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
    ")?;

    // doc_nodes.node_type
    let has_node_type: bool = conn
        .prepare("SELECT node_type FROM doc_nodes LIMIT 0")
        .is_ok();
    if !has_node_type {
        conn.execute_batch("ALTER TABLE doc_nodes ADD COLUMN node_type TEXT NOT NULL DEFAULT 'section';")?;
    }

    // edges.source_kind / target_kind
    let has_source_kind: bool = conn
        .prepare("SELECT source_kind FROM edges LIMIT 0")
        .is_ok();
    if !has_source_kind {
        conn.execute_batch("
            ALTER TABLE edges ADD COLUMN source_kind TEXT NOT NULL DEFAULT 'symbol';
            ALTER TABLE edges ADD COLUMN target_kind TEXT NOT NULL DEFAULT 'symbol';
        ")?;
    }

    Ok(())
}
