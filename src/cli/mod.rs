use clap::Subcommand;
use crate::embedding::Embedder;
#[derive(Subcommand)]
pub enum Command {
    Index { #[arg(default_value=".")] path: String, #[arg(long)] branch: Option<String>, #[arg(long)] repo: Option<String>, #[arg(long)] parent: Option<String> },
    Status { #[arg(long)] repo: Option<String> },
    Pull { source: String },
    Push { #[arg(default_value="auto")] source: String },
    SwitchBranch { name: String },
    Mcp, Check,
    Branch { #[command(subcommand)] cmd: BranchCmd },
}
#[derive(Subcommand)]
pub enum BranchCmd {
    SetAlias { alias: String, branch: String, #[arg(long)] desc: Option<String>, #[arg(long)] repo: Option<String> },
    ListAliases { #[arg(long)] repo: Option<String> },
}
pub async fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Index { path, branch, repo, parent } => {
            let branch = branch.unwrap_or_else(|| crate::indexer::git::current_branch(&path).unwrap_or_else(|| "unknown".into()));
            let repo = repo.unwrap_or_else(|| std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or("default".into()));
            println!("Indexing {} (branch={}, repo={})...", path, branch, repo);
            let data_dir = crate::config::Config::data_dir()?;
            let db_path = data_dir.join(format!("{}.rag.db", repo));
            let conn = crate::storage::open(&db_path.to_string_lossy())?;
            crate::storage::migrate(&conn)?;
            let result = if let Some(ref p) = parent {
                crate::indexer::smart::smart_index_with_parent(&conn, &path, &repo, &branch, p)?
            } else {
                crate::indexer::smart::smart_index(&conn, &path, &repo, &branch)?
            };
            println!("Done: {} files, {} symbols", result.files_scanned, result.symbols_new);
            if result.symbols_inherited > 0 { println!("  {} inherited from {}", result.symbols_inherited, result.inherited_from.as_deref().unwrap_or("parent")); }
            if let Some(ref from) = result.from_commit { println!("  delta: {}..{}", &from[..8.min(from.len())], result.head_commit.as_deref().map(|h|&h[..8]).unwrap_or("?")); }
            // also index docs + link to code
            index_docs(&conn, &path, &repo);
            index_includes(&conn, &path, &repo);
            let embedder = crate::embedding::TextEmbedder;
            match crate::embedding::link_docs_to_symbols(&conn, &embedder, &repo, 0.05) {
                Ok(n) => println!("  Linked: {} doc-symbol pairs", n),
                Err(e) => eprintln!("  Link warning: {}", e),
            }
        }
        Command::Branch { cmd } => match cmd {
            BranchCmd::SetAlias { alias, branch, desc, repo } => {
                let repo = repo.unwrap_or("default".into());
                let dd = crate::config::Config::data_dir()?;
                let dbp = dd.join(format!("{}.rag.db", repo));
                let c = crate::storage::open(&dbp.to_string_lossy())?;
                crate::storage::migrate(&c)?;
                c.execute("INSERT OR REPLACE INTO branch_glossary(repo,branch_name,alias,description) VALUES(?1,?2,?3,?4)", rusqlite::params![repo,branch,alias,desc])?;
                println!("Alias: {} -> {}", alias, branch);
            }
            BranchCmd::ListAliases { repo } => {
                let repo = repo.unwrap_or("default".into());
                let dd = crate::config::Config::data_dir()?;
                let dbp = dd.join(format!("{}.rag.db", repo));
                let c = crate::storage::open(&dbp.to_string_lossy())?;
                let mut s = c.prepare("SELECT alias,branch_name,description FROM branch_glossary WHERE repo=?1 ORDER BY alias")?;
                let r = s.query_map(rusqlite::params![repo], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,Option<String>>(2)?)))?;
                let mut n=0;
                for x in r { let (a,b,d)=x?; println!("  {} -> {} | {}", a,b,d.unwrap_or_default()); n+=1; }
                if n==0 { println!("No aliases. Add: codeloom branch set-alias <alias> <branch> --repo <repo>"); }
            }
        },
        Command::Status {..} => println!("Status..."),
        Command::Pull {..} => println!("Pull..."),
        Command::Push {..} => println!("Push..."),
        Command::SwitchBranch {..} => println!("Switch..."),
        Command::Mcp => crate::mcp::serve().await?,
        Command::Check => {
            println!("CodeLoom v{}", env!("CARGO_PKG_VERSION"));
            println!("Binary: {:?}", std::env::current_exe().unwrap_or_default());
        }
    }
    Ok(())
}
fn index_docs(conn: &rusqlite::Connection, dir: &str, repo: &str) {
    let mut doc_count = 0;
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !["md","rst"].contains(&ext) { continue; }
        let ps = p.to_string_lossy();
        if ps.contains("/tests/")||ps.contains("/build/")||ps.contains("/.git/") { continue; }
        if let Ok(content) = std::fs::read_to_string(ps.as_ref()) {
            // Store branch glossary entries
            let entries = crate::doc::glossary::parse_branch_glossary(&content);
            if !entries.is_empty() { println!("  Glossary: {} entries from {}", entries.len(), ps); }
            // Store as doc_nodes for embedding-based search
            match crate::doc::index_markdown(conn, &ps, &content, repo) {
                Ok(n) => { doc_count += n; }
                Err(e) => { eprintln!("  Warning: doc index {}: {}", ps, e); }
            }
        }
    }
    if doc_count > 0 { println!("  Docs: {} sections from {}", doc_count, dir); }
}

/// Extract #include relations from source files and store as edges
pub fn index_includes(conn: &rusqlite::Connection, dir: &str, repo: &str) -> usize {
    let re = regex::Regex::new(r#"#include\s*[<"]([^>"]+)[>"]"#).unwrap();
    let mut count = 0;
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !["h","hpp","hxx","cpp","cxx","cc","c"].contains(&ext) { continue; }
        if let Ok(content) = std::fs::read_to_string(p) {
            for cap in re.captures_iter(&content) {
                let included = cap[1].to_string();
                conn.execute(
                    "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, source_repo) VALUES (0, 0, ?1, ?2)",
                    rusqlite::params![format!("includes:{}", included), repo],
                ).ok();
                count += 1;
            }
        }
    }
    if count > 0 { println!("  Includes: {} edges", count); }
    count
}