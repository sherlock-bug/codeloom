use clap::Subcommand;
use crate::embedding::Embedder;

/// 团队代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱
#[derive(Subcommand)]
pub enum Command {
    /// 索引代码库：扫描 C++/Python/Java/TS/Go 代码，提取符号和边
    ///
    /// 示例:
    ///   codeloom index .                    # 索引当前目录
    ///   codeloom index /path/to/repo --repo myrepo
    ///   codeloom index . --branch feature-x --parent main
    Index {
        /// 要索引的目录或文件路径
        #[arg(default_value=".")]
        path: String,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
        /// 仓库标识名（默认取目录名）
        #[arg(long)]
        repo: Option<String>,
        /// 从指定分支继承符号（替代自动检测 merge-base）
        #[arg(long)]
        parent: Option<String>,
    },

    /// 查看索引状态：符号数、边数、文档数、DB 大小
    Status {
        /// 仓库标识名（默认 "default"）
        #[arg(long)]
        repo: Option<String>,
    },

    /// 拉取团队共享知识库（需要配置 remote 地址）
    Pull {
        /// 远程 DB 路径或 URL
        source: String,
    },

    /// 推送本地知识库到团队共享存储
    Push {
        /// 目标路径（默认 "auto"）
        #[arg(default_value="auto")]
        source: String,
    },

    /// 切换活跃分支
    SwitchBranch {
        /// 目标分支名
        name: String,
    },

    /// 启动 MCP JSON-RPC 服务（供 OpenCode 等 AI 编码助手调用）
    Mcp,

    /// 检查运行环境：显示版本号和二进制路径
    Check,

    /// 管理分支惯用叫法映射
    #[command(subcommand)]
    Branch(BranchCmd),

    /// 自动更新到最新 GitHub Release 版本
    Update,
}

/// 分支别名管理
#[derive(Subcommand)]
pub enum BranchCmd {
    /// 添加分支别名映射（如 23B → release/2023-B）
    SetAlias {
        /// 惯用叫法
        alias: String,
        /// 实际分支名
        branch: String,
        /// 可选描述
        #[arg(long)]
        desc: Option<String>,
        /// 仓库标识名
        #[arg(long)]
        repo: Option<String>,
    },
    /// 列出当前仓库的所有分支别名
    ListAliases {
        /// 仓库标识名
        #[arg(long)]
        repo: Option<String>,
    },
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
        Command::Branch(cmd) => match cmd {
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
        Command::Status { repo } => {
            let repo = repo.as_deref().unwrap_or("default");
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() {
                println!("No index found for repo '{}'. Run: codeloom index <path> --repo {}", repo, repo);
                return Ok(());
            }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let syms: i64 = conn.query_row("SELECT COUNT(*) FROM symbols WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
            let edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
            let docs: i64 = conn.query_row("SELECT COUNT(*) FROM doc_nodes WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
            let links: i64 = conn.query_row("SELECT COUNT(*) FROM doc_code_links WHERE doc_node_id IN (SELECT id FROM doc_nodes WHERE repo=?1)", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
            let resolved: i64 = conn.query_row("SELECT COUNT(*) FROM edges WHERE target_id!=0", [], |r| r.get(0)).unwrap_or(0);
            let meta = std::fs::metadata(&dbp).ok();
            println!("Repo: {}", repo);
            println!("  Symbols: {}  |  Edges: {} (resolved: {} / {:.0}%)  |  Docs: {}", syms, edges, resolved, if edges>0 {resolved as f64/edges as f64*100.0}else{0.0}, docs);
            println!("  Doc-code links: {}", links);
            if let Some(m) = meta { println!("  DB size: {:.1} MB", m.len() as f64 / 1_048_576.0); }
        }
        Command::Pull {..} => println!("Pull..."),
        Command::Push {..} => println!("Push..."),
        Command::SwitchBranch {..} => println!("Switch..."),
        Command::Mcp => crate::mcp::serve().await?,
        Command::Check => {
            println!("CodeLoom v{}", env!("CARGO_PKG_VERSION"));
            println!("Binary: {:?}", std::env::current_exe().unwrap_or_default());
        }
        Command::Update => do_update(),
    }
    Ok(())
}
fn index_docs(conn: &rusqlite::Connection, dir: &str, repo: &str) {
    let mut doc_count = 0;
    let ignore_patterns = crate::ignore::load_patterns(dir);
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !["md","rst"].contains(&ext) { continue; }
        let ps = p.to_string_lossy();
        if ps.contains("/.git/") { continue; }
        if crate::ignore::is_ignored(&ps, &ignore_patterns) { continue; }
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
    let ignore_patterns = crate::ignore::load_patterns(dir);
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !["h","hpp","hxx","cpp","cxx","cc","c"].contains(&ext) { continue; }
        if crate::ignore::is_ignored(&p.to_string_lossy(), &ignore_patterns) { continue; }
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

// ── Self-update ────────────────────────────────────────────────────────

const GITHUB_RELEASES_API: &str = "https://api.github.com/repos/sherlock-bug/codeloom/releases/latest";
const GITHUB_DOWNLOAD: &str = "https://github.com/sherlock-bug/codeloom/releases/download";
const GHPROXY_DOWNLOAD: &str = "https://ghproxy.net/https://github.com/sherlock-bug/codeloom/releases/download";

fn do_update() {
    let current_ver = env!("CARGO_PKG_VERSION");
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => { eprintln!("Cannot determine binary path: {}", e); return; }
    };

    println!("CodeLoom v{} — checking for updates...", current_ver);

    // Query GitHub API for latest release
    let latest_tag = match get_latest_tag() {
        Some(t) => t,
        None => { eprintln!("Failed to check GitHub. Check network or try again later."); return; }
    };

    let latest_ver = latest_tag.trim_start_matches('v');
    if latest_ver == current_ver {
        println!("Already up to date (v{})", current_ver);
        return;
    }
    println!("New version: {} → upgrading from v{}", latest_tag, current_ver);

    // Detect platform
    let platform = match detect_platform() {
        Some(p) => p,
        None => { eprintln!("Unsupported platform. Use --from-source install instead."); return; }
    };

    let binary_name = format!("codeloom-{}", platform);
    let download_url = format!("{}/{}/{}", GITHUB_DOWNLOAD, latest_tag, binary_name);
    let mirror_url = format!("{}/{}/{}", GHPROXY_DOWNLOAD, latest_tag, binary_name);

    // Download via ghproxy, fallback to direct
    let tmp = match current_exe.parent() {
        Some(dir) => dir.join(".codeloom.tmp"),
        None => { eprintln!("Cannot determine install path"); return; }
    };

    println!("Downloading {}...", binary_name);
    let mut downloaded = false;
    for (name, url) in [("ghproxy", &mirror_url), ("direct", &download_url)] {
        let status = std::process::Command::new("curl")
            .args(["-sSL", "--connect-timeout", "10", "--max-time", "300", "-o"])
            .arg(&tmp)
            .arg(url)
            .status();
        if let Ok(s) = status {
            if s.success() {
                downloaded = true;
                if name == "direct" { println!("  Downloaded via direct (ghproxy unavailable)"); }
                break;
            }
        }
        eprintln!("  {} failed, trying {}...", name, if name=="ghproxy" {"direct"}else{""});
    }

    if !downloaded {
        eprintln!("Download failed. Try manually:");
        eprintln!("  curl -sSL {} -o codeloom", mirror_url);
        return;
    }

    // Verify and replace
    match std::fs::metadata(&tmp) {
        Ok(meta) if meta.len() > 1_000_000 => {
            // Set executable permission
            #[cfg(unix)] { let _ = std::process::Command::new("chmod").args(["+x"]).arg(&tmp).status(); }
            if let Err(e) = std::fs::rename(&tmp, &current_exe) {
                eprintln!("Cannot replace binary: {}. Try: mv {} {}", e, tmp.display(), current_exe.display());
            } else {
                println!("Updated to {} ✓", latest_tag);
                println!("Run 'codeloom check' to verify.");
            }
        }
        Ok(meta) => {
            eprintln!("Download too small ({} bytes). Aborting.", meta.len());
        }
        Err(e) => {
            eprintln!("Download verification failed: {}", e);
        }
    }
}

fn get_latest_tag() -> Option<String> {
    let output = std::process::Command::new("curl")
        .args(["-sS", "--connect-timeout", "10", "--max-time", "15",
               "-H", "Accept: application/vnd.github+json",
               GITHUB_RELEASES_API])
        .output().ok()?;
    if !output.status.success() { return None; }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    json.get("tag_name")?.as_str().map(|s| s.to_string())
}

fn detect_platform() -> Option<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("linux", "x86_64") => Some("linux-x86_64".into()),
        ("linux", "aarch64") => Some("linux-arm64".into()),
        ("macos", "x86_64") => Some("darwin-x86_64".into()),
        ("macos", "aarch64") => Some("darwin-arm64".into()),
        _ => None,
    }
}