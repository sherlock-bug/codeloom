//! 向量搜索噪声标定模块
//!
//! BM25 使用硬阈值 0.5（分数有绝对语义），向量搜索需要标定（相似度分布取决于模型和索引内容）。
//! 每次 index 后自动标定向量搜索的噪声基线，搜索时用 z-score 过滤低置信度结果。

use anyhow::Context;
use rusqlite::Connection;

// ── 噪声探针 ──────────────────────────────────────────────
// KNN 总能返回最近邻 → 这些探针在任意代码库都能命中结果

const NOISE_PROBES: &[&str] = &[
    "morning coffee afternoon tea",
    "地铁换乘站周末出行",
    "今天晚饭吃什么好呢",
    "春天来了桃花开了",
    "xyxxy foobarbaz quuxzot kzmwptn",
];

// ── 噪声基线 ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NoiseProfile {
    pub noise_mean: f64,
    pub noise_std: f64,
    pub noise_ceiling: f64,
    pub samples: usize,
    pub model: String,
    pub calibrated_at: String,
}

// ── config.db 持久化 ──────────────────────────────────────

fn config_db_path() -> std::path::PathBuf {
    crate::config::Config::data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("config.db")
}

fn open_config_db() -> anyhow::Result<Connection> {
    let path = config_db_path();
    let conn = Connection::open(&path)
        .with_context(|| format!("无法打开 config.db: {}", path.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS noise_profile (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            noise_mean REAL NOT NULL,
            noise_std REAL NOT NULL,
            noise_ceiling REAL NOT NULL,
            samples INTEGER NOT NULL,
            model TEXT NOT NULL,
            calibrated_at TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

pub fn save_noise_profile(profile: &NoiseProfile) -> anyhow::Result<()> {
    let conn = open_config_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO noise_profile (id, noise_mean, noise_std, noise_ceiling, samples, model, calibrated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            profile.noise_mean,
            profile.noise_std,
            profile.noise_ceiling,
            profile.samples as i64,
            profile.model,
            profile.calibrated_at,
        ],
    )?;
    Ok(())
}

pub fn load_noise_profile() -> Option<NoiseProfile> {
    let conn = open_config_db().ok()?;
    conn.query_row(
        "SELECT noise_mean, noise_std, noise_ceiling, samples, model, calibrated_at
         FROM noise_profile WHERE id = 1",
        [],
        |row| {
            Ok(NoiseProfile {
                noise_mean: row.get(0)?,
                noise_std: row.get(1)?,
                noise_ceiling: row.get(2)?,
                samples: row.get::<_, i64>(3)? as usize,
                model: row.get(4)?,
                calibrated_at: row.get(5)?,
            })
        },
    )
    .ok()
}

// ── 标定入口 ──────────────────────────────────────────────

/// 执行向量搜索噪声标定。
///
/// 找符号数最多的仓库，对 5 条探针各跑 KNN Top 5，用 similarity 分布计算基线。
/// 标定前先删旧基线（避免旧数据干扰）。
pub fn calibrate() -> anyhow::Result<NoiseProfile> {
    // 先清旧基线
    let _ = std::fs::remove_file(config_db_path());

    let embedder = crate::embedding::get_embedder()
        .context("embedder not initialized")?;

    // Embed all probes at once
    let probe_embs = embedder.embed_batch(NOISE_PROBES)
        .context("failed to embed noise probes")?;

    let repos = crate::query::repo::list_repos();
    if repos.is_empty() {
        return Err(anyhow::anyhow!("没有已索引仓库，无法标定"));
    }

    let data_dir = crate::config::Config::data_dir()?;

    // 找符号数最多的仓库
    let mut best_repo = String::new();
    let mut best_branch = String::new();
    let mut max_syms: i64 = 0;

    for repo in &repos {
        let db_path = data_dir.join(format!("{}.rag.db", repo));
        if !db_path.exists() { continue; }
        let conn = match crate::storage::open(&db_path.to_string_lossy()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let syms: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))
            .unwrap_or(0);
        if syms <= max_syms { continue; }

        let branch: Option<String> = (|| -> Option<String> {
            let mut stmt = conn
                .prepare("SELECT branch_name FROM branches WHERE branch_name IS NOT NULL LIMIT 1")
                .ok()?;
            let mut rows = stmt.query_map([], |r| r.get(0)).ok()?;
            rows.next()?.ok()
        })();
        if let Some(b) = branch {
            max_syms = syms;
            best_repo = repo.clone();
            best_branch = b;
        }
    }

    if best_repo.is_empty() {
        return Err(anyhow::anyhow!("所有仓库均无符号，无法标定"));
    }

    let db_path = data_dir.join(format!("{}.rag.db", best_repo));
    let conn = crate::storage::open(&db_path.to_string_lossy())?;

    // 确保 vec0 扩展已加载
    if !crate::storage::vector::try_load(&conn) {
        return Err(anyhow::anyhow!("vec0 extension not loaded"));
    }

    let sym_table = format!("symbol_name_vec_{}", best_repo.replace('-', "_"));
    let mut all_sims: Vec<f64> = Vec::new();
    let mut knn_errors: Vec<String> = Vec::new();

    for (i, emb) in probe_embs.iter().enumerate() {
        match crate::storage::vector::knn_search(&conn, &sym_table, emb, 5) {
            Ok(rows) => {
                for (_, dist) in rows {
                    let sim = 1.0 / (1.0 + dist as f64);
                    all_sims.push(sim);
                }
            }
            Err(e) => {
                knn_errors.push(format!("probe[{}] '{}': {}", i, NOISE_PROBES[i], e));
            }
        }
    }
    drop(conn);

    if all_sims.is_empty() {
        let detail = if knn_errors.is_empty() {
            "所有探针返回 0 行".to_string()
        } else {
            format!("{} 个 KNN 错误: {}", knn_errors.len(), knn_errors.join("; "))
        };
        return Err(anyhow::anyhow!("向量探针未返回任何结果: {}", detail));
    }

    let n = all_sims.len() as f64;
    let mean: f64 = all_sims.iter().sum::<f64>() / n;
    let variance = all_sims.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    let ceiling = mean + 2.0 * std;

    let model = crate::config::Config::load()
        .ok()
        .and_then(|c| c.embedding)
        .map(|e| e.model)
        .unwrap_or_else(|| "unknown".to_string());

    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    Ok(NoiseProfile {
        noise_mean: mean,
        noise_std: std,
        noise_ceiling: ceiling,
        samples: all_sims.len(),
        model,
        calibrated_at: now,
    })
}
