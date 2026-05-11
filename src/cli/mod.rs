use clap::Subcommand;
use crate::{log_info, log_error, log_warn};

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

    /// 启动 MCP JSON-RPC 服务（供 OpenCode 等 AI 编码助手调用）
    /// --http :8080 启动 HTTP 远程模式
    Mcp {
        #[arg(long)]
        http: Option<String>,
    },

    /// 检查运行环境：显示版本号和二进制路径
    Check,

    /// 管理分支惯用叫法映射
    #[command(subcommand)]
    Branch(BranchCmd),

    /// 生成 Shell 补全脚本（bash/zsh/fish）
    ///
    /// 用法: source <(codeloom completion bash)
    Completion {
        /// Shell 类型
        shell: String,
    },

    /// 自动更新到最新 GitHub Release 版本
    Update,

    /// 清理索引数据：删除指定仓库/分支/全部数据
    /// ...
    ///
    /// 示例:
    ///   codeloom clean --all                 # 清空所有数据
    ///   codeloom clean --repo myrepo         # 删除某仓库
    ///   codeloom clean --repo myrepo --branch feature-x  # 删除某分支
    Clean {
        /// 清空所有仓库的所有数据（需确认）
        #[arg(long)]
        all: bool,
        /// 目标仓库名
        #[arg(long)]
        repo: Option<String>,
        /// 目标分支名（需同时指定 --repo）
        #[arg(long, requires = "repo")]
        branch: Option<String>,
    },

    /// 列出所有已索引的仓库
    ListRepos,

    /// 列出指定仓库的所有已索引分支
    ListBranches {
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
    },

    /// 精确 BM25 关键词搜索（符号名/注释/文档/文件），不涉及向量语义。语义搜索用 `codeloom semantic` 命令
    Search {
        /// 搜索关键词或功能描述
        query: String,
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
        /// 返回结果数
        #[arg(long, default_value = "10")]
        limit: usize,
        /// 按符号类型过滤 (function, method, class, struct, enum, etc.)
        #[arg(long)]
        kind: Option<String>,
    },

    /// 向量语义搜索 — 用自然语言描述功能查找相关符号。不依赖关键词匹配，理解语义
    Semantic {
        /// 语义搜索描述（自然语言）
        query: String,
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
        /// 返回结果数
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// 查看仓库架构全貌（符号按类型分布）
    Overview {
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
    },

    /// 按名称模糊搜索符号
    ListSymbols {
        /// 搜索模式（SQL LIKE）
        pattern: String,
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
        /// 返回结果数
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// 查看节点全部信息：定义、注释、所有关联边
    Inspect {
        /// 符号完整名称（C++ 类方法用 ClassName::methodName）
        name: String,
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
    },

    /// 分析函数调用关系
    CallGraph {
        /// 符号完整名称
        name: String,
        /// 仓库标识名（默认自动检测）
        #[arg(long)]
        repo: Option<String>,
        /// Git 分支名（默认自动检测）
        #[arg(long)]
        branch: Option<String>,
        /// 方向：callers（谁调用了它）或 callees（它调用了谁）
        #[arg(long, default_value = "callers")]
        direction: String,
        /// 最大递归深度
        #[arg(long, default_value = "3")]
        max_depth: usize,
    },
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
// ── Repository & branch auto-detection ─────────────────────────────────

/// Auto-detect repo name from current directory.
/// Logic mirrors Index command: git repos use directory name, non-git returns empty string.
fn autodetect_repo() -> String {
    let cwd = std::env::current_dir().ok()
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_default();
    if cwd.is_empty() { return String::new(); }
    // If current dir is a git repo, use directory name
    if crate::indexer::git::current_branch(".").is_some() {
        std::path::Path::new(&cwd)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".into())
    } else {
        String::new()
    }
}

/// Auto-detect git branch from current directory.
/// Returns Some(branch) if in a git repo, None otherwise.
fn autodetect_branch() -> Option<String> {
    crate::indexer::git::current_branch(".")
}

pub async fn run(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Index { path, branch, repo, parent } => {
            let git_branch = crate::indexer::git::current_branch(&path);
            let is_git = git_branch.is_some();
            let branch = branch.unwrap_or_else(|| git_branch.unwrap_or_else(|| "unknown".into()));
            let repo = repo.unwrap_or_else(|| {
                // Non-git folder without explicit --repo → use "" (global visibility)
                if !is_git {
                    return String::new();
                }
                // When path is ".", use actual current directory name
                let p = if path == "." || path == "./" {
                    std::env::current_dir().ok().map(|d| d.to_string_lossy().to_string()).unwrap_or(path.clone())
                } else {
                    path.clone()
                };
                std::path::Path::new(&p).file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "default".into())
            });
            let t0 = std::time::Instant::now();
            log_info!("index", "开始索引 {} @ {}:{}", path, repo, branch);
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
            let t1 = t0.elapsed();
            println!("Done: {} files, {} symbols ({:.1}s parse+db)", result.files_scanned, result.symbols_new, t1.as_secs_f64());
            if result.symbols_inherited > 0 { println!("  {} inherited from {}", result.symbols_inherited, result.inherited_from.as_deref().unwrap_or("parent")); }
            if let Some(ref from) = result.from_commit { println!("  delta: {}..{}", &from[..8.min(from.len())], result.head_commit.as_deref().map(|h|&h[..8]).unwrap_or("?")); }
            // also index docs
            index_docs(&conn, &path, &repo);
            let t2 = t0.elapsed();
            index_includes(&conn, &path, &repo);
            match crate::storage::fts::fill_all_fts(&conn, &repo) {
                Ok(n) => { eprintln!("  FTS5: {} entries indexed", n); log_info!("index", "FTS5 完成: {} entries", n); }
                Err(e) => { eprintln!("  FTS5 warning: {}", e); log_error!("index", "FTS5 失败: {}", e); }
            }
            let t3 = t0.elapsed();
            // Symbol + Doc + File vectors — run in spawn_blocking to avoid reqwest::blocking tokio conflict
            let (sym_n, doc_n, file_n) = tokio::task::spawn_blocking(move || -> anyhow::Result<(usize, usize, usize)> {
                let embedder = crate::embedding::get_embedder()?;
                let (sym_n, doc_n) = crate::embedding::index_vectors(&conn, &repo, embedder)?;
                let file_n = crate::embedding::index_file_vectors(&conn, &repo, embedder)?;
                Ok((sym_n, doc_n, file_n))
            }).await??;
            if sym_n + doc_n + file_n > 0 {
                eprintln!("  Vectors: {} symbols, {} docs, {} files", sym_n, doc_n, file_n);
            }
            let t4 = t0.elapsed();
            eprintln!("  ⏱  parse+db: {:.1}s | docs+fts: {:.1}s | vectors: {:.1}s | total: {:.1}s",
                t1.as_secs_f64(), (t3-t2).as_secs_f64(), (t4-t3).as_secs_f64(), t4.as_secs_f64());

            // 向量搜索噪声标定（spawn_blocking 避免 reqwest::blocking 与 tokio 冲突）
            match tokio::task::spawn_blocking(crate::calib::calibrate).await {
                Ok(Ok(p)) => {
                    eprintln!("  Noise calib: μ={:.4} σ={:.4} z=2.0 → ceiling={:.4} ({} samples, model={})",
                        p.noise_mean, p.noise_std, p.noise_ceiling, p.samples, p.model);
                    if let Err(e) = crate::calib::save_noise_profile(&p) {
                        log_error!("index", "标定保存失败: {}", e);
                    }
                }
                Ok(Err(e)) => {
                    log_error!("index", "向量噪声标定失败: {}", e);
                    eprintln!("  Noise calibration skipped: {}", e);
                }
                Err(join_err) => {
                    log_error!("index", "标定线程 panic: {}", join_err);
                    eprintln!("  Noise calibration skipped: thread panicked");
                }
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
            let dd = crate::config::Config::data_dir()?;
            let repo = repo.unwrap_or_else(|| {
                let detected = autodetect_repo();
                if !detected.is_empty() && dd.join(format!("{}.rag.db", detected)).exists() {
                    return detected;
                }
                // Try "default" as fallback
                if dd.join("default.rag.db").exists() { return "default".into(); }
                detected
            });
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() {
                println!("No index found for repo '{}'. Run: codeloom index <path> --repo {}", repo, repo);
                return Ok(());
            }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let syms: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE repo=?1 AND node_type='sym'", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
            let edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
            let docs: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE repo=?1 AND node_type='doc'", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
            let resolved: i64 = conn.query_row("SELECT COUNT(*) FROM edges WHERE target_id!=0", [], |r| r.get(0)).unwrap_or(0);
            let sym_name_table = format!("symbol_name_vec_{}", repo.replace('-', "_"));
            let sym_comment_table = format!("symbol_comment_vec_{}", repo.replace('-', "_"));
            let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));
            let vec_syms_name: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {sym_name_table}"), [], |r| r.get(0)).unwrap_or(0);
            let vec_syms_comment: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {sym_comment_table}"), [], |r| r.get(0)).unwrap_or(0);
            let vec_docs: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {doc_table}"), [], |r| r.get(0)).unwrap_or(0);
            let fts5_all: i64 = conn.query_row("SELECT COUNT(*) FROM fts5_all", [], |r| r.get(0)).unwrap_or(0);
            let meta = std::fs::metadata(&dbp).ok();
            println!("Repo: {}", repo);
            println!("  Symbols: {}  |  Edges: {} (resolved: {} / {:.0}%)  |  Docs: {}", syms, edges, resolved, if edges>0 {resolved as f64/edges as f64*100.0}else{0.0}, docs);
            println!("  Vectors: {} name + {} comment + {} docs indexed", vec_syms_name, vec_syms_comment, vec_docs);
            println!("  FTS5: {} entries indexed (unified)", fts5_all);
            if let Some(m) = meta { println!("  DB size: {:.1} MB", m.len() as f64 / 1_048_576.0); }
        }
        Command::Mcp { http } => {
            if let Some(addr) = http {
                let addr = if addr.starts_with(':') { format!("0.0.0.0{}", addr) } else { addr };
                crate::mcp::serve_http(&addr).await?;
            } else {
                crate::mcp::serve_stdio().await?;
            }
        }
        Command::Check => {
            println!("CodeLoom v{}", env!("CARGO_PKG_VERSION"));
            println!("Binary: {:?}", std::env::current_exe().unwrap_or_default());
            println!();

            // 1. Data directory
            let data_dir = match crate::config::Config::data_dir() {
                Ok(d) => {
                    println!("[OK] Data dir: {}", d.display());
                    d
                }
                Err(e) => {
                    println!("[FAIL] Data dir: {}", e);
                    return Ok(());
                }
            };

            // 2. vec0 extension (statically compiled)
            {
                let test_db = data_dir.join("_check_test.db");
                let result = (|| -> Result<_, String> {
                    let conn = rusqlite::Connection::open(&test_db).map_err(|e| format!("can't open test DB: {e}"))?;
                    conn.execute_batch(
                        "CREATE VIRTUAL TABLE IF NOT EXISTS _vec0_check_ USING vec0(embedding FLOAT[1]);
                         DROP TABLE IF EXISTS _vec0_check_;",
                    )
                    .map_err(|e| format!("vec0 not available: {e}"))
                })();
                match result {
                    Ok(()) => println!("[OK]  vec0: available (statically compiled)"),
                    Err(e) => println!("[FAIL] vec0: {e}"),
                }
                let _ = std::fs::remove_file(&test_db);
            }

            // 3. FTS5 (built into SQLite)
            {
                let test_db = data_dir.join("_check_test.db");
                if let Ok(conn) = rusqlite::Connection::open(&test_db) {
                    let result = conn.execute_batch(
                        "CREATE VIRTUAL TABLE IF NOT EXISTS _fts5_test USING fts5(x); DROP TABLE IF EXISTS _fts5_test;"
                    );
                    if result.is_ok() {
                        println!("[OK]  FTS5: built-in SQLite full-text search");
                    } else {
                        println!("[FAIL] FTS5: SQLite compiled without FTS5 support");
                    }
                }
                let _ = std::fs::remove_file(test_db);
            }

            // 4. clang (for C/C++ AST parsing)
            {
                let has_clang = std::process::Command::new("which")
                    .arg("clang")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if has_clang {
                    println!("[OK]  clang: found on PATH");
                } else {
                    println!("[MISS] clang: not found. C/C++ indexing needs clang++.");
                    println!("       Install: sudo apt install clang  (Ubuntu/Debian)");
                    println!("       Or: sudo dnf install clang    (Fedora)");
                }
            }

            // 5. python3 (for Clang AST filter pipeline)
            {
                let has_python3 = std::process::Command::new("which")
                    .arg("python3")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if has_python3 {
                    println!("[OK]  python3: found on PATH");
                } else {
                    println!("[MISS] python3: not found. C/C++ indexing needs python3.");
                    println!("       Install: sudo apt install python3  (Ubuntu/Debian)");
                }
            }

            // 6. clang_filter.py (for Clang AST pipeline)
            {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                let filter_path = std::path::Path::new(&home).join(".codeloom/scripts/clang_filter.py");
                if filter_path.exists() {
                    println!("[OK]  clang_filter.py: {}", filter_path.display());
                } else {
                    println!("[MISS] clang_filter.py: not found at {}", filter_path.display());
                    println!("       Reinstall with: curl -sSL <url> | bash");
                }
            }

            // 7. Embedding (API-based)
            let config = crate::config::Config::load().unwrap_or_default();
            match &config.embedding {
                Some(cfg) => {
                    println!("[OK]  Embed API: {} (model: {})", cfg.api_base, cfg.model);
                    match crate::embedding::get_embedder() {
                        Ok(embedder) => match embedder.embed("test") {
                            Ok(v) => println!("       Embed test: {} dims OK", v.len()),
                            Err(e) => println!("[WARN] Embed test failed: {}", e),
                        },
                        Err(e) => println!("[WARN] Embedder init failed: {}", e),
                    }
                }
                None => {
                    println!("[MISS] Embed API: not configured.");
                    println!("       Add 'embedding' section to ~/.codeloom/config.yaml:");
                    println!("         embedding:");
                    println!("           api_base: \"http://your-llm:8080/v1\"");
                    println!("           model: \"bge-m3\"");
                }
            }

            // 5. Indexed repos
            let repos = crate::query::repo::list_repos();
            if repos.is_empty() {
                println!();
                println!("[INFO] No indexed repos. Run: codeloom index <path> --repo <name> --branch <branch>");
            } else {
                println!();
                println!("Indexed repos:");
                for repo in &repos {
                    let db_path = data_dir.join(format!("{}.rag.db", repo));
                    let size = std::fs::metadata(&db_path).map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0);
                    let syms: i64 = if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                        conn.query_row("SELECT COUNT(*) FROM nodes WHERE repo=?1 AND node_type='sym'", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0)
                    } else { 0 };
                    let display = if repo.is_empty() { "(global)" } else { &repo };
                    println!("  {:20}  {:>6} symbols  {:>5.1} MB", display, syms, size);
                }
            }

            println!();
            println!("MCP tools: codeloom mcp  (9 tools, use codeloom_search for hybrid BM25+vector search)");
        }
        Command::Update => do_update(),
        Command::Clean { all, repo, branch } => do_clean(all, repo, branch),
        Command::ListRepos => {
            let repos = crate::query::repo::list_repos();
            if repos.is_empty() {
                println!("No indexed repos found in ~/.codeloom/");
            } else {
                println!("Indexed repos:");
                if let Ok(dd) = crate::config::Config::data_dir() {
                    for repo in &repos {
                        let db_path = dd.join(format!("{}.rag.db", repo));
                        let size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
                        println!("  {:20}  {:>8.1} MB", repo, size as f64 / 1_048_576.0);
                    }
                } else {
                    for repo in &repos {
                        println!("  {}", repo);
                    }
                }
            }
        }
        Command::ListBranches { repo } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() {
                println!("Repo '{}' not found. Run 'codeloom list-repos' to see available repos.", repo);
                return Ok(());
            }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let mut stmt = conn.prepare("SELECT DISTINCT branch_name FROM branches WHERE branch_name IS NOT NULL ORDER BY branch_name")?;
            let branches: Vec<String> = stmt.query_map([], |r| r.get(0))?.flatten().collect();
            if branches.is_empty() {
                println!("Repo '{}': no branches indexed.", repo);
            } else {
                println!("Repo '{}' branches:", repo);
                for b in &branches {
                    let sym_count: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM nodes n JOIN branches b2 ON b2.node_id=n.id WHERE n.node_type='sym' AND b2.branch_name=?1",
                        rusqlite::params![b],
                        |r| r.get(0),
                    ).unwrap_or(0);
                    println!("  {:20}  {} symbols", b, sym_count);
                }
            }
        }
        Command::Search { query, repo, branch, limit, kind } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            let branch = branch.or_else(autodetect_branch).unwrap_or_else(|| "main".into());
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() { println!("Repo '{}' not found.", repo); return Ok(()); }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let q = query.clone();
            let kind_filter = kind.clone();
            let results = tokio::task::spawn_blocking(move || {
                crate::query::search::bm25_precise_search(&conn, &query, &repo, &branch, limit, kind_filter.as_deref(), false)
            }).await??;
            {
                let results = results;
                if results.is_empty() {
                    println!("(no results)");
                } else {
                    log_info!("search", "搜索 \"{}\" → {} results", q, results.len());
                    println!("搜索 \"{}\") ({}条):", q, results.len());
                    for r in &results {
                        let typ = if r.hit_type == "code" && !r.kind.is_empty() {
                            r.kind.clone()
                        } else {
                            r.hit_type.clone()
                        };
                        let comment: String = r.snippet.chars().take(120).collect();
                        println!("  [{:16}] {:40}  @ {}",
                            typ, r.name, comment);
                    }
                }
            }
        }
        Command::Semantic { query, repo, branch, limit } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            let branch = branch.or_else(autodetect_branch).unwrap_or_else(|| "main".into());
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() { println!("Repo '{}' not found.", repo); return Ok(()); }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let q = query.clone();
            let results = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
                let embedder = crate::embedding::get_embedder()?;
                let emb = embedder.embed(&query)?;
                crate::query::search::vector_semantic_search(&conn, &emb, &repo, &branch, limit, false)
            }).await??;
            if results.is_empty() {
                println!("(no results)");
            } else {
                log_info!("semantic", "语义搜索 \"{}\" → {} results", q, results.len());
                println!("语义搜索 \"{}\" ({}条):", q, results.len());
                for r in &results {
                    let typ = if r.hit_type == "code" && !r.kind.is_empty() {
                        r.kind.clone()
                    } else {
                        r.hit_type.clone()
                    };
                    let comment: String = r.snippet.chars().take(120).collect();
                    println!("  [{:16}] {:40}  @ {:.4} {}",
                        typ, r.name, r.score, comment);
                }
            }
        }
        Command::Overview { repo, branch } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            let branch = branch.or_else(autodetect_branch).unwrap_or_else(|| "main".into());
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() { println!("Repo '{}' not found.", repo); return Ok(()); }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let syms: i64 = conn.query_row(
                "SELECT COUNT(*) FROM nodes n JOIN branches b ON b.node_id=n.id WHERE b.repo=?1 AND b.branch_name=?2 AND n.node_type='sym'",
                rusqlite::params![repo, branch], |r| r.get(0)).unwrap_or(0);
            let edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
            let docs: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name=?2) AND node_type='doc'",
                rusqlite::params![repo, branch], |r| r.get(0)).unwrap_or(0);
            println!("=== {} (branch={}) ===", repo, branch);
            println!("Symbols: {}  |  Edges: {}  |  Docs: {}\n", syms, edges, docs);
            println!("Symbols by kind:");
            let mut stmt = conn.prepare(
                "SELECT kind, COUNT(*) FROM nodes n JOIN branches b ON b.node_id=n.id WHERE b.repo=?1 AND b.branch_name=?2 AND n.node_type='sym' GROUP BY kind ORDER BY COUNT(*) DESC"
            )?;
            let kinds: Vec<(String, i64)> = stmt.query_map(rusqlite::params![repo, branch], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?)))?.flatten().collect();
            for (kind, count) in &kinds {
                let pct = if syms > 0 { *count as f64 / syms as f64 * 100.0 } else { 0.0 };
                println!("  {:12}: {:5} ({:.1}%)", kind, count, pct);
            }
        }
        Command::ListSymbols { pattern, repo, branch, limit } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            let branch = branch.or_else(autodetect_branch).unwrap_or_else(|| "main".into());
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() { println!("Repo '{}' not found.", repo); return Ok(()); }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let like = format!("%{}%", pattern);
            let sql = "SELECT name, kind, file_path, line_start FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.repo=?1 AND n.name LIKE ?2 AND n.node_type='sym' AND b.branch_name=?3 ORDER BY name LIMIT ?4";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params![repo, like, branch, limit as i64], |r| {
                Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?))
            })?;
            println!("Symbols matching '{}' in {} (branch={}):", pattern, repo, branch);
            let mut count = 0;
            for row in rows.flatten() {
                count += 1;
                println!("  [{:10}] {:40}  @ {}:{}", row.1, row.0, &row.2[..60.min(row.2.len())], row.3);
            }
            if count == 0 { println!("  (none)"); }
        }
        Command::Inspect { name, repo, branch } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            let branch = branch.or_else(autodetect_branch).unwrap_or_else(|| "main".into());
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() { println!("Repo '{}' not found.", repo); return Ok(()); }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            // Fetch symbol with all fields
            let sql = "SELECT n.id, n.name, n.kind, n.file_path, n.line_start, CAST(json_extract(n.attrs,'$.line_end') AS INTEGER), json_extract(n.attrs,'$.signature'), json_extract(n.attrs,'$.parent_class'), json_extract(n.attrs,'$.namespace'), json_extract(n.attrs,'$.language'), n.content FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.repo=?1 AND n.name=?2 AND n.node_type='sym' AND b.branch_name=?3 LIMIT 5";
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(rusqlite::params![repo, name, branch], |r| {
                Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?,
                    r.get::<_,String>(3)?, r.get::<_,i64>(4)?, r.get::<_,i64>(5)?,
                    r.get::<_,Option<String>>(6)?, r.get::<_,Option<String>>(7)?,
                    r.get::<_,Option<String>>(8)?, r.get::<_,Option<String>>(9)?,
                    r.get::<_,Option<String>>(10)?))
            })?;
            let mut found = false;
            for (_i, row) in rows.flatten().enumerate() {
                found = true;
                let (sid, sname, kind, file, lstart, lend, sig, parent, ns, lang, doc) = row;
                println!("╔══ {} ══╗", sname);
                println!("║ Kind:     {}", kind);
                if let Some(ref l) = lang { println!("║ Language: {}", l); }
                if let Some(ref n) = ns { println!("║ Namespace: {}", n); }
                if let Some(ref p) = parent { println!("║ In class: {}", p); }
                println!("║ File:     {}:{}-{}", file, lstart, lend);
                if let Some(ref s) = sig { println!("║ Signature: {}", s); }
                if let Some(ref d) = doc {
                    let doc_trimmed = d.trim();
                    if !doc_trimmed.is_empty() {
                        println!("╟── Documentation ──");
                        for line in doc_trimmed.lines() {
                            println!("║ {}", line);
                        }
                    }
                }
                // ── Edges ──
                let mut edge_count = 0usize;
                if let Ok(mut e_stmt) = conn.prepare(
                    "SELECT e.edge_type, s.name, s.kind, s.file_path, s.line_start
                     FROM edges e LEFT JOIN nodes n ON (
                        (e.source_id=?1 AND n.id=e.target_id) OR (e.target_id=?1 AND n.id=e.source_id)
                     ) WHERE (e.source_id=?1 OR e.target_id=?1) AND n.id IS NOT NULL AND n.node_type='sym'
                     ORDER BY e.edge_type LIMIT 200") {
                    if let Ok(e_rows) = e_stmt.query_map(rusqlite::params![sid], |r| {
                        Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?,
                            r.get::<_,String>(3)?, r.get::<_,Option<i64>>(4)?))
                    }) {
                        // Group by edge type category
                        use std::collections::BTreeMap;
                        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
                        for row in e_rows.flatten() {
                            let (etype, ename, ekind, efile, eline) = row;
                            let loc = if let Some(l) = eline { format!(" @ {}:{}", &efile[..60.min(efile.len())], l) } else { String::new() };
                            let entry = format!("  {} [{}]{}", ename, ekind, loc);
                            let cat = categorize_edge(&etype).to_string();
                            groups.entry(cat).or_default().push(entry);
                        }
                        for (cat, entries) in &groups {
                            if !entries.is_empty() {
                                println!("╟── {} ──", cat);
                                for e in entries.iter().take(15) { println!("{}", e); }
                                if entries.len() > 15 { println!("  ... and {} more", entries.len() - 15); }
                                edge_count += entries.len();
                            }
                        }
                    }
                }
                if edge_count == 0 { println!("╟── (no edges)"); }
                println!("╚{}", "═".repeat(sname.len() + 4));
            }
            if !found { println!("Symbol '{}' not found.", name); }
        }
        Command::CallGraph { name, repo, branch, direction, max_depth } => {
            let repo = repo.unwrap_or_else(autodetect_repo);
            let branch = branch.or_else(autodetect_branch).unwrap_or_else(|| "main".into());
            if repo.is_empty() { println!("No repo detected. Specify --repo or run from a git repo."); return Ok(()); }
            let dd = crate::config::Config::data_dir()?;
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() { println!("Repo '{}' not found.", repo); return Ok(()); }
            let conn = crate::storage::open(&dbp.to_string_lossy())?;
            let ids: Vec<i64> = conn.prepare(
                "SELECT n.id FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.repo=?1 AND n.name=?2 AND n.node_type='sym' AND b.branch_name=?3"
            )?.query_map(rusqlite::params![repo, name, branch], |r| r.get(0))?.flatten().collect();
            if ids.is_empty() {
                println!("Symbol '{}' not found in {} (branch={}).", name, repo, branch);
                return Ok(());
            }
            println!("Call graph for '{}' ({}):", name, direction);
            let mut visited = std::collections::HashSet::new();
            for &root_id in &ids {
                visited.insert(root_id);
                println!("  * {} (id={})", name, root_id);
                traverse_calls_cli(&conn, root_id, &direction, max_depth, 1, &mut visited, &branch);
            }
        }
        _ => {}, // Completion handled in main.rs
    }
    Ok(())
}

fn traverse_calls_cli(conn: &rusqlite::Connection, sym_id: i64, direction: &str,
    max_depth: usize, depth: usize, visited: &mut std::collections::HashSet<i64>, branch: &str,
) {
    if depth > max_depth { return; }
    let prefix = "  ".repeat(depth + 1);
    let query = match direction {
        "callees" => format!("SELECT e.target_id, e.edge_type FROM edges e WHERE e.source_id={} AND e.target_id!=0 AND e.edge_type LIKE 'calls:%'", sym_id),
        _ => format!("SELECT e.source_id, e.edge_type FROM edges e WHERE e.target_id={} AND e.edge_type LIKE 'calls:%'", sym_id),
    };
    if let Ok(mut stmt) = conn.prepare(&query) {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
            for row in rows.flatten() {
                let (other_id, edge_type) = row;
                if visited.contains(&other_id) {
                    let repeated = conn.query_row("SELECT name FROM nodes WHERE id=?1 AND node_type='sym'", rusqlite::params![other_id], |r| r.get::<_,String>(0)).unwrap_or_default();
                    println!("{}{} {} (already shown)", prefix, '→', repeated);
                    continue;
                }
                visited.insert(other_id);
                let other_name = conn.query_row(
                    "SELECT n.name FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.id=?1 AND n.node_type='sym' AND b.branch_name=?2",
                    rusqlite::params![other_id, branch], |r| r.get::<_,String>(0)
                ).unwrap_or_default();
                let called = edge_type.strip_prefix("calls:").unwrap_or(&edge_type);
                println!("{}{} {} (calls:{})", prefix, '→', other_name, called);
                if depth < max_depth {
                    traverse_calls_cli(conn, other_id, direction, max_depth, depth + 1, visited, branch);
                }
            }
        }
    }
}

/// Group raw edge types into display categories
fn categorize_edge(etype: &str) -> &str {
    if etype.starts_with("calls:") { return "📞 Calls"; }
    if etype.starts_with("inherits:") { return "🧬 Inherits"; }
    if etype.starts_with("overrides:") { return "🔄 Overrides"; }
    if etype.starts_with("contains:") { return "📦 Contains"; }
    if etype.starts_with("param_type:") { return "📥 Parameters"; }
    if etype.starts_with("returns:") { return "📤 Returns"; }
    if etype.starts_with("field_type:") { return "⚡ Fields"; }
    "🔗 Other"
}

fn index_docs(conn: &rusqlite::Connection, dir: &str, repo: &str) {
    let mut doc_count = 0;
    let ignore_patterns = crate::ignore::load_patterns(dir);
    let supported = ["md", "rst", "xlsx", "xls", "xlsm", "docx", "pdf", "xml"];
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let p = entry.path();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !supported.contains(&ext) { continue; }
        let ps = p.to_string_lossy();
        if ps.contains("/.git/") { continue; }
        if crate::ignore::is_ignored(&ps, &ignore_patterns) { continue; }

        // Read file bytes
        let bytes = match std::fs::read(p) {
            Ok(b) => b,
            Err(e) => { eprintln!("  Warning: read {}: {}", ps, e); continue; }
        };

        // Glossary extraction (md/rst only)
        if ext == "md" || ext == "rst" {
            if let Ok(content) = String::from_utf8(bytes.clone()) {
                let entries = crate::doc::glossary::parse_branch_glossary(&content);
                if !entries.is_empty() { println!("  Glossary: {} entries from {}", entries.len(), ps); }
            }
        }

        // Parse and write
        match crate::doc::parse_document(ext, &ps, &bytes) {
            Ok(sections) => {
                match crate::doc::write_doc_sections(conn, repo, &ps, ext, &sections) {
                    Ok(n) => {
                        doc_count += n;
                        if n > 10 { println!("  {}: {} nodes", ps, n); }
                    }
                    Err(e) => eprintln!("  Warning: write {}: {}", ps, e),
                }
            }
            Err(e) => eprintln!("  Warning: parse {}: {}", ps, e),
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
        if let Ok(content) = crate::util::read_file_smart(p) {
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
            // Validate: must be an ELF binary (not an HTML error page)
            if let Ok(buf) = std::fs::read(&tmp) {
                if buf.len() < 4 || &buf[..4] != b"\x7fELF" {
                    eprintln!("Download corrupt (not an ELF binary). Aborting.");
                    let _ = std::fs::remove_file(&tmp);
                    return;
                }
            }
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

// ── Clean / Reset ──────────────────────────────────────────────────────

fn do_clean(all: bool, repo: Option<String>, branch: Option<String>) {
    let repo = repo.or_else(|| {
        let r = autodetect_repo();
        if r.is_empty() { None } else { Some(r) }
    });
    let dd = match crate::config::Config::data_dir() {
        Ok(d) => d,
        Err(e) => { eprintln!("Error: {}", e); return; }
    };

    if all {
        // ── Nuke everything ──────────────────────────────────────
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(&dd) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if name.ends_with(".rag.db") || name.ends_with(".rag.db-wal") || name.ends_with(".rag.db-shm")
                    || name.ends_with(".rag.overlay.db")
                {
                    if std::fs::remove_file(&p).is_ok() { count += 1; }
                }
            }
        }
        log_info!("clean", "清理全部: {} file(s)", count);
        println!("Cleaned all data: {} file(s) removed from {}", count, dd.display());
        return;
    }

    let repo = match repo {
        Some(r) => r,
        None => {
            eprintln!("Specify --all, --repo, or --repo --branch. See: codeloom clean --help");
            return;
        }
    };

    let db_path = dd.join(format!("{}.rag.db", repo));
    if !db_path.exists() {
        println!("No data for repo '{}' ({} not found)", repo, db_path.display());
        return;
    }

    if let Some(ref branch) = branch {
        // ── Remove specific branch ────────────────────────────────
        let conn = match crate::storage::open(&db_path.to_string_lossy()) {
            Ok(c) => c,
            Err(e) => { eprintln!("Error opening DB: {}", e); return; }
        };
        // Delete branch-specific data from all tables that have branch_name
        let mut total = 0;
        for (table, col) in [
            ("branches", "branch_name"),
            ("git_index_state", "branch_name"),
            ("branch_glossary", "branch_name"),
            ("nodes", "branch_name"),
        ] {
            let sql = format!("DELETE FROM {} WHERE {} = ?1 AND (repo = ?2 OR repo IS NULL OR repo = '')", table, col);
            if let Ok(n) = conn.execute(&sql, rusqlite::params![branch, repo]) {
                total += n;
            }
        }
        // Also drop vec tables for this repo to force full vector regeneration on re-index
        let vec_prefix = format!("%vec_{}%", repo.replace('-', "_"));
        if let Ok(mut stmt) = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE ?1") {
            if let Ok(rows) = stmt.query_map(rusqlite::params![vec_prefix], |r| r.get::<_,String>(0)) {
                for row in rows.flatten() {
                    if conn.execute_batch(&format!("DROP TABLE IF EXISTS [{}]", row)).is_ok() {
                        total += 1;
                    }
                }
            }
        }
        println!("Repo '{}', branch '{}': removed {} row(s)", repo, branch, total);
    } else {
        // ── Remove entire repo ────────────────────────────────────
        let mut removed = 0;
        for suffix in ["", "-wal", "-shm"] {
            let p = dd.join(format!("{}.rag.db{}", repo, suffix));
            if p.exists() && std::fs::remove_file(&p).is_ok() { removed += 1; }
        }
        // Also remove overlay
        let overlay = dd.join(format!("{}.rag.overlay.db", repo));
        if overlay.exists() && std::fs::remove_file(&overlay).is_ok() { removed += 1; }
        println!("Repo '{}': removed {} file(s)", repo, removed);
    }
}