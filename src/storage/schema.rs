use rusqlite::Connection;

pub fn run(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY AUTOINCREMENT, repo TEXT NOT NULL DEFAULT 'default',
            name TEXT NOT NULL, kind TEXT NOT NULL,
            content_hash TEXT NOT NULL, file_path TEXT NOT NULL,
            line_start INTEGER NOT NULL DEFAULT 0, line_end INTEGER NOT NULL DEFAULT 0,
            language TEXT, signature TEXT, parent_class TEXT, namespace TEXT,
            sid TEXT, access TEXT DEFAULT '', is_virtual INTEGER DEFAULT 0,
            is_definition INTEGER DEFAULT 1, is_external INTEGER DEFAULT 0,
            template_args TEXT,
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

        CREATE VIRTUAL TABLE IF NOT EXISTS fts5_sym USING fts5(name, file_path, signature, definition, kind);
        CREATE VIRTUAL TABLE IF NOT EXISTS fts5_doc USING fts5(title, section_path, content);
    ")?;
    // New indexes for Clang columns (separate batch — symbols table must exist first)
    conn.execute_batch("
        CREATE INDEX IF NOT EXISTS idx_sym_sid ON symbols(sid);
        CREATE INDEX IF NOT EXISTS idx_sym_external ON symbols(is_external);
    ")?;
    // v0.5.0 migrations: multi-format doc + image support
    migrate_v5(conn)?;
    // v0.5.x migration: fts5_sym column extension (definition + kind)
    migrate_v6(conn)?;
    // v0.6.0 migration: comment + file nodes + doc chunking
    migrate_v7(conn)?;
    migrate_v8(conn)?;
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

/// Rebuild fts5_sym with 5 columns (adds definition + kind).
/// FTS5 doesn't support ALTER, so we DROP and recreate.
fn migrate_v6(conn: &Connection) -> anyhow::Result<()> {
    // Check if fts5_sym already has the 'definition' column
    let has_definition: bool = {
        let test_sql = "SELECT definition FROM fts5_sym LIMIT 0";
        conn.prepare(test_sql).is_ok()
    };

    if !has_definition {
        // Drop old 3/4-column FTS5 and recreate with 5 columns
        conn.execute_batch("
            DROP TABLE IF EXISTS fts5_sym;
            CREATE VIRTUAL TABLE fts5_sym USING fts5(name, file_path, signature, definition, kind);
        ")?;
    }

    Ok(())
}

/// v0.6.0 migration: doc_comment, parent_id, file nodes, doc_comment in FTS5
fn migrate_v7(conn: &Connection) -> anyhow::Result<()> {
    // 1. symbols.doc_comment
    let has_doc_comment: bool = conn
        .prepare("SELECT doc_comment FROM symbols LIMIT 0")
        .is_ok();
    if !has_doc_comment {
        conn.execute_batch("ALTER TABLE symbols ADD COLUMN doc_comment TEXT NOT NULL DEFAULT '';")?;
    }

    // 2. doc_nodes.parent_id
    let has_parent_id: bool = conn
        .prepare("SELECT parent_id FROM doc_nodes LIMIT 0")
        .is_ok();
    if !has_parent_id {
        conn.execute_batch("
            ALTER TABLE doc_nodes ADD COLUMN parent_id INTEGER REFERENCES doc_nodes(id);
            CREATE INDEX IF NOT EXISTS idx_doc_parent ON doc_nodes(parent_id);
        ")?;
    }

    // 3. files table
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY,
            repo TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_type TEXT NOT NULL CHECK(file_type IN ('code', 'doc')),
            summary TEXT NOT NULL DEFAULT '',
            content_hash TEXT NOT NULL DEFAULT '',
            branch_name TEXT DEFAULT 'main',
            UNIQUE(repo, file_path, branch_name)
        );
        CREATE INDEX IF NOT EXISTS idx_files_repo ON files(repo);
    ")?;

    // 4. fts5_files FTS5 index
    let has_fts_files: bool = {
        let test_sql = "SELECT file_path FROM fts5_files LIMIT 0";
        conn.prepare(test_sql).is_ok()
    };
    if !has_fts_files {
        conn.execute_batch("
            CREATE VIRTUAL TABLE fts5_files USING fts5(file_path, summary);
        ")?;
    }

    // 5. Rebuild fts5_sym with doc_comment column
    let has_doc_comment_fts: bool = {
        let test_sql = "SELECT doc_comment FROM fts5_sym LIMIT 0";
        conn.prepare(test_sql).is_ok()
    };
    if !has_doc_comment_fts {
        conn.execute_batch("
            DROP TABLE IF EXISTS fts5_sym;
            CREATE VIRTUAL TABLE fts5_sym USING fts5(name, file_path, signature, kind, doc_comment);
        ")?;
    }

    Ok(())
}


/// v0.7.0 migration: Clang parser columns (sid, access, is_virtual, is_definition, is_external, template_args)
fn migrate_v8(conn: &Connection) -> anyhow::Result<()> {
    let has_sid: bool = conn.prepare("SELECT sid FROM symbols LIMIT 0").is_ok();
    if !has_sid {
        conn.execute_batch("
            ALTER TABLE symbols ADD COLUMN sid TEXT;
            ALTER TABLE symbols ADD COLUMN access TEXT DEFAULT '';
            ALTER TABLE symbols ADD COLUMN is_virtual INTEGER DEFAULT 0;
            ALTER TABLE symbols ADD COLUMN is_definition INTEGER DEFAULT 1;
            ALTER TABLE symbols ADD COLUMN is_external INTEGER DEFAULT 0;
            ALTER TABLE symbols ADD COLUMN template_args TEXT;
            CREATE INDEX IF NOT EXISTS idx_sym_sid ON symbols(sid);
            CREATE INDEX IF NOT EXISTS idx_sym_external ON symbols(is_external);
        ")?;
    }
    // Rebuild FTS5 with new columns
    let has_sid_fts: bool = conn.prepare("SELECT sid FROM fts5_sym LIMIT 0").is_ok();
    if !has_sid_fts {
        conn.execute_batch("
            DROP TABLE IF EXISTS fts5_sym;
            CREATE VIRTUAL TABLE fts5_sym USING fts5(name, file_path, signature, kind, doc_comment, sid, template_args);
        ")?;
    }
    Ok(())
}