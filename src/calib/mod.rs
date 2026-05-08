//! 噪声标定模块
//!
//! 使用已索引仓库进行噪声标定，计算搜索噪声基线，
//! 用于在搜索时自动过滤低置信度结果。

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
    pub noise_mean: f64,
    pub noise_std: f64,
    pub noise_ceiling: f64,
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

pub fn noise_ceiling() -> Option<f64> {
    load_noise_profile().map(|p| p.noise_ceiling)
}

// ── 标定 ──────────────────────────────────────────────────

/// 执行噪声标定（目标 <5s）
///
/// 取符号数最多的仓库，跑 3 条噪声探针各 Top 5
pub fn calibrate() -> anyhow::Result<NoiseProfile> {
    // 先清旧基线，避免过滤干扰标定
    let _ = std::fs::remove_file(config_db_path());

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

    let mut all_scores: Vec<f64> = Vec::new();
    for &probe in NOISE_PROBES {
        if let Ok(results) =
            crate::query::search::hybrid_search(&conn, probe, &best_repo, &best_branch, 5, None)
        {
            all_scores.extend(results.iter().map(|r| r.score));
        }
    }
    drop(conn);

    if all_scores.is_empty() {
        return Err(anyhow::anyhow!("噪声探针未返回任何结果"));
    }

    let n = all_scores.len() as f64;
    let mean: f64 = all_scores.iter().sum::<f64>() / n;
    let variance = all_scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    let ceiling = mean + 2.5 * std;

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
        samples: all_scores.len(),
        model,
        calibrated_at: now,
    })
}

// calib 模块无独立单元测试。noise_profile 的持久化通过集成测试间接覆盖。
