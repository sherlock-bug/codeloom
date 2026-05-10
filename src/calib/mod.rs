//! 噪声标定模块
//!
//! 使用已索引仓库进行噪声标定，计算搜索噪声基线，
//! 用于在搜索时自动过滤低置信度结果。
//! BM25 和向量通道各自独立标定。

use anyhow::Context;
use rusqlite::Connection;

// ── 噪声探针 ──────────────────────────────────────────────

/// 噪声探针：3 条中文 + 1 条英文 + 1 条随机字符串，保证与任何代码库无重叠
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
    pub channel: String,   // "bm25" | "vector" | "hybrid"(legacy)
    pub top1_mean: f64,
    pub top1_std: f64,
    pub samples: usize,
    pub model: String,
    pub calibrated_at: String,
}

// ── config.db 全局配置 ────────────────────────────────────

fn config_db_path() -> std::path::PathBuf {
    crate::config::Config::data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("config.db")
}

fn open_config_db() -> anyhow::Result<Connection> {
    let path = config_db_path();
    let conn = Connection::open(&path)
        .with_context(|| format!("无法打开 config.db: {}", path.display()))?;

    // Create table with channel column
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS noise_profile (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel TEXT NOT NULL,
            top1_mean REAL NOT NULL,
            top1_std REAL NOT NULL,
            samples INTEGER NOT NULL,
            model TEXT NOT NULL,
            calibrated_at TEXT NOT NULL,
            UNIQUE(channel)
        );",
    )?;

    // Migration: add channel column if missing (old schema)
    let has_channel: bool = conn
        .prepare("SELECT channel FROM noise_profile LIMIT 0")
        .is_ok();
    if !has_channel {
        conn.execute_batch(
            "ALTER TABLE noise_profile ADD COLUMN channel TEXT NOT NULL DEFAULT 'hybrid';",
        )?;
    }

    Ok(conn)
}

pub fn save_noise_profile(profile: &NoiseProfile) -> anyhow::Result<()> {
    let conn = open_config_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO noise_profile (channel, top1_mean, top1_std, samples, model, calibrated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            profile.channel,
            profile.top1_mean,
            profile.top1_std,
            profile.samples as i64,
            profile.model,
            profile.calibrated_at,
        ],
    )?;
    Ok(())
}

pub fn load_noise_profile(channel: &str) -> Option<NoiseProfile> {
    let conn = open_config_db().ok()?;
    conn.query_row(
        "SELECT channel, top1_mean, top1_std, samples, model, calibrated_at
         FROM noise_profile WHERE channel = ?1",
        rusqlite::params![channel],
        |row| {
            Ok(NoiseProfile {
                channel: row.get(0)?,
                top1_mean: row.get(1)?,
                top1_std: row.get(2)?,
                samples: row.get::<_, i64>(3)? as usize,
                model: row.get(4)?,
                calibrated_at: row.get(5)?,
            })
        },
    )
    .ok()
}

/// 按通道获取噪音基线（用于搜索过滤）
pub fn noise_profile(channel: &str) -> Option<NoiseProfile> {
    load_noise_profile(channel)
}

// ── 内部辅助 ──────────────────────────────────────────────

/// 找符号数最多的仓库（标定公用）
fn find_largest_repo() -> anyhow::Result<(String, String, String)> {
    let repos = crate::query::repo::list_repos();
    if repos.is_empty() {
        return Err(anyhow::anyhow!("没有已索引仓库，无法标定"));
    }

    let data_dir = crate::config::Config::data_dir()?;
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
    Ok((best_repo, best_branch, db_path.to_string_lossy().to_string()))
}

fn compute_noise_profile(scores: &[f64], channel: &str) -> anyhow::Result<NoiseProfile> {
    if scores.is_empty() {
        return Err(anyhow::anyhow!("噪声探针未返回任何结果"));
    }

    let n = scores.len() as f64;
    let mean: f64 = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();

    let model = crate::config::Config::load()
        .ok()
        .and_then(|c| c.embedding)
        .map(|e| e.model)
        .unwrap_or_else(|| "unknown".to_string());

    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    Ok(NoiseProfile {
        channel: channel.to_string(),
        top1_mean: mean,
        top1_std: std,
        samples: scores.len(),
        model,
        calibrated_at: now,
    })
}

// ── 标定入口 ──────────────────────────────────────────────

/// 执行 BM25 通道噪音标定（5 条探针 × bm25_precise_search）
pub fn calibrate_bm25() -> anyhow::Result<NoiseProfile> {
    let (repo, branch, db_path) = find_largest_repo()?;
    let conn = crate::storage::open(&db_path)?;

    let mut top1_scores: Vec<f64> = Vec::new();
    for &probe in NOISE_PROBES {
        if let Ok(results) =
            crate::query::search::bm25_precise_search(&conn, probe, &repo, &branch, 5, None, true)
        {
            if let Some(top) = results.first() {
                top1_scores.push(top.score);
            }
        }
    }
    drop(conn);

    compute_noise_profile(&top1_scores, "bm25")
}

/// 执行向量通道噪音标定（5 条探针 × vector_semantic_search）
pub fn calibrate_vector() -> anyhow::Result<NoiseProfile> {
    let (repo, branch, db_path) = find_largest_repo()?;
    let conn = crate::storage::open(&db_path)?;

    let embedder = crate::embedding::get_embedder()
        .map_err(|e| anyhow::anyhow!("嵌入模型不可用: {}", e))?;

    let mut top1_scores: Vec<f64> = Vec::new();
    for &probe in NOISE_PROBES {
        if let Ok(emb) = embedder.embed(probe) {
            if let Ok(results) =
                crate::query::search::vector_semantic_search(&conn, &emb, &repo, &branch, 5, true)
            {
                if let Some(top) = results.first() {
                    top1_scores.push(top.score);
                }
            }
        }
    }
    drop(conn);

    compute_noise_profile(&top1_scores, "vector")
}

/// 全量标定：依次执行 BM25 和向量通道标定
pub fn calibrate() -> anyhow::Result<Vec<NoiseProfile>> {
    let mut profiles = Vec::new();

    match calibrate_bm25() {
        Ok(p) => {
            save_noise_profile(&p)?;
            profiles.push(p);
        }
        Err(e) => eprintln!("  [SKIP] BM25 标定失败: {}", e),
    }

    match calibrate_vector() {
        Ok(p) => {
            save_noise_profile(&p)?;
            profiles.push(p);
        }
        Err(e) => eprintln!("  [SKIP] 向量标定失败: {}", e),
    }

    if profiles.is_empty() {
        return Err(anyhow::anyhow!("所有通道标定均失败"));
    }

    Ok(profiles)
}
