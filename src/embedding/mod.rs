// Embedding module: candle-based semantic embeddings with TextEmbedder fallback.
use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::OnceLock;
use rusqlite::Connection;

/// Embedding vector (384-dim for bge-small-zh, variable-dim for Jaccard)
pub type Embedding = Vec<f32>;

/// Trait for embedding providers
pub trait Embedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding>;
    fn dimension(&self) -> usize;
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32;
}

// ── Cosine similarity ─────────────────────────────────────────────────

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let dot: f32 = a.iter().zip(b).map(|(x,y)| x*y).sum();
    let na: f32 = a.iter().map(|x| x*x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x*x).sum::<f32>().sqrt();
    if na < 1e-10 || nb < 1e-10 { return 0.0; }
    (dot / (na * nb)).max(0.0).min(1.0)
}

// ── CandleEmbedder (bge-small-zh, 384-dim) ────────────────────────────

static CANDLE_LOADED: OnceLock<Result<(candle_transformers::models::bert::BertModel, tokenizers::Tokenizer, candle_core::Device), String>> = OnceLock::new();

pub struct CandleEmbedder;

impl CandleEmbedder {
    /// Resolve model directory: CARGO_MANIFEST_DIR first, then binary-relative, then HOME.
    fn resolve_model_dir() -> std::path::PathBuf {
        // 0. Check compile-time CARGO_MANIFEST_DIR (works for tests + dev)
        let manifest_models = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("models").join("bge-small-zh");
        if manifest_models.join("pytorch_model.bin").exists() {
            return manifest_models;
        }
        // 1. Try relative to binary (for installed deployment: /usr/local/bin + models/)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(bin_dir) = exe.parent() {
                let candidate = bin_dir.join("models").join("bge-small-zh");
                if candidate.join("pytorch_model.bin").exists() {
                    return candidate;
                }
                if let Some(up) = bin_dir.parent() {
                    let candidate2 = up.join("models").join("bge-small-zh");
                    if candidate2.join("pytorch_model.bin").exists() {
                        return candidate2;
                    }
                }
            }
        }
        // 2. Fallback: ~/.codeloom/models/bge-small-zh/
        dirs::home_dir()
            .unwrap_or_default()
            .join(".codeloom")
            .join("models")
            .join("bge-small-zh")
    }

    /// Check if model files exist on disk
    pub fn model_available() -> bool {
        let dir = Self::resolve_model_dir();
        dir.join("pytorch_model.bin").exists()
            && dir.join("config.json").exists()
            && dir.join("tokenizer.json").exists()
    }

    fn get_or_load() -> Result<&'static (candle_transformers::models::bert::BertModel, tokenizers::Tokenizer, candle_core::Device), &'static str> {
        CANDLE_LOADED.get_or_init(|| {
                let dir = Self::resolve_model_dir();
            let device = candle_core::Device::Cpu;
            
            // Load config
            let config_path = dir.join("config.json");
            let config_str = std::fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config.json: {}", e))?;
            let config: candle_transformers::models::bert::Config = 
                serde_json::from_str(&config_str)
                .map_err(|e| format!("Failed to parse config.json: {}", e))?;
            
                // Load model weights from PyTorch format
                let model_path = dir.join("pytorch_model.bin");
                let vb = candle_nn::VarBuilder::from_pth(&model_path, candle_core::DType::F32, &device)
                    .map_err(|e| format!("Failed to load model weights: {}", e))?;
            let model = candle_transformers::models::bert::BertModel::load(vb, &config)
                .map_err(|e| format!("Failed to init BertModel: {}", e))?;
            
            // Load tokenizer
            let tokenizer_path = dir.join("tokenizer.json");
            let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| format!("Failed to load tokenizer: {}", e))?;
            
            Ok((model, tokenizer, device))
        }).as_ref().map_err(|e| e.as_str())
    }
}

impl Embedder for CandleEmbedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding> {
        let (model, tokenizer, device) = Self::get_or_load()
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // Tokenize
        let encoding = tokenizer.encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization error: {}", e))?;
        let token_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        if token_ids.len() > 512 {
            return Err(anyhow::anyhow!("Input too long (max 512 tokens)"));
        }
        
        let input_ids = candle_core::Tensor::new(&token_ids[..], device)?;
        let input_ids = input_ids.unsqueeze(0)?; // [1, seq_len]
        let token_type_ids = input_ids.zeros_like()?;
        let attention_mask = input_ids.ones_like()?;
        
        // Forward pass
        let output = model.forward(&input_ids, &token_type_ids, Some(&attention_mask))?;
        
        // Mean pooling: average all token embeddings (BGE models)
        let seq_len = output.dims()[1] as usize;
        let pooled = output.sum(1)?;
        let pooled = (pooled / seq_len as f64)?; // mean over seq_len → [1, 512]
        let cls = pooled.get(0)?; // → [512]
        let vec: Vec<f32> = cls.to_vec1()?;
        
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-10 {
            Ok(vec.iter().map(|x| x / norm).collect())
        } else {
            Ok(vec)
        }
    }
    
    fn dimension(&self) -> usize { 384 }
    
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        cosine_similarity(a, b)
    }
}

// ── TextEmbedder (Jaccard token overlap, fallback) ───────────────────

pub struct TextEmbedder;
impl Embedder for TextEmbedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding> {
        Ok(tokenize(text))
    }
    fn dimension(&self) -> usize { 384 }
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        jaccard_from_tokens(a, b)
    }
}

// ── Tokenization helpers ──────────────────────────────────────────────

fn tokenize(text: &str) -> Vec<f32> {
    let tokens: HashSet<u64> = text
        .split(|c: char| !c.is_alphanumeric())
        .map(|t| t.trim().to_lowercase())
        .filter(|t| t.len() >= 2 && !STOP_WORDS.contains(&&t[..]))
        .map(|t| {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            t.hash(&mut h);
            std::hash::Hash::hash(&t, &mut h);
            h.finish()
        })
        .collect();
    tokens.into_iter().map(|h| h as f32).collect()
}

fn jaccard_from_tokens(a: &[f32], b: &[f32]) -> f32 {
    let set_a: HashSet<u64> = a.iter().map(|&x| x as u64).collect();
    let set_b: HashSet<u64> = b.iter().map(|&x| x as u64).collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 { return 0.0; }
    intersection as f32 / union as f32
}

static STOP_WORDS: &[&str] = &[
    "the","and","for","this","that","with","from","have","are","was",
    "not","but","all","can","has","had","been","will","each","its",
    "int","void","const","auto","bool","char","double","float","long",
    "short","unsigned","signed","static","inline","virtual","override",
    "public","private","protected","class","struct","enum","return",
];


// ── Vector indexing (sqlite-vec) ──────────────────────────────────────

/// Index all symbols and docs for a repo into vec0 vector tables.
/// Returns (symbol_count, doc_count) or vec0 is not available.
/// Returns (symbol_count, doc_count) or vec0 is not available.
/// Incremental: skips symbols/docs that already have vectors in vec0 tables.
pub fn index_vectors(conn: &rusqlite::Connection, repo: &str, embedder: &dyn Embedder) -> anyhow::Result<(usize, usize)> {
    if !crate::storage::vector::try_load(conn) {
        return Ok((0, 0));
    }
    crate::storage::vector::create_tables(conn, repo)?;
    let sym_table = format!("symbol_vec_{}", repo.replace('-', "_"));
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));

    // Collect existing vec0 rowids (for incremental skip)
    let existing_sym_ids: std::collections::HashSet<i64> = {
        let mut s = std::collections::HashSet::new();
        if let Ok(mut stmt) = conn.prepare(&format!("SELECT rowid FROM {sym_table}")) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() { s.insert(r); }
            }
        }
        s
    };
    let existing_doc_ids: std::collections::HashSet<i64> = {
        let mut s = std::collections::HashSet::new();
        if let Ok(mut stmt) = conn.prepare(&format!("SELECT rowid FROM {doc_table}")) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() { s.insert(r); }
            }
        }
        s
    };

    // Index symbol vectors (only new ones)
    let mut sym_count = 0; let mut sym_skipped = 0; let mut sym_total = 0;
    if let Ok(mut stmt) = conn.prepare("SELECT id, name, kind, definition FROM symbols WHERE repo=?1") {
        let mut batch = Vec::new();
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            sym_total = all.len();
            let mut processed = 0;
            let report_every = if sym_total > 100 { sym_total / 10 } else { 10 };
            for row in all {
                if existing_sym_ids.contains(&row.0) { sym_skipped += 1; processed += 1; continue; }
                let text = format!("{} {} {}", row.2, row.1, row.3);
                let emb = embedder.embed(&text).unwrap_or_default();
                if !emb.is_empty() {
                    batch.push((row.0, emb));
                }
                if batch.len() >= 50 {
                    sym_count += crate::storage::vector::insert_vectors(conn, &sym_table, &batch.iter().map(|(id, v)| (*id, v.as_slice())).collect::<Vec<_>>())?;
                    batch.clear();
                }
                processed += 1;
                if processed % report_every == 0 {
                    eprint!("\r  Vectors: {}/{} symbols ({}%)...", processed, sym_total, processed * 100 / sym_total);
                }
            }
            if sym_total > 0 {
                eprint!("\r  Vectors: {}/{} symbols done.", processed, sym_total);
                eprintln!();
            }
        }
        if !batch.is_empty() {
            sym_count += crate::storage::vector::insert_vectors(conn, &sym_table, &batch.iter().map(|(id, v)| (*id, v.as_slice())).collect::<Vec<_>>())?;
        }
    }

    // Index doc vectors (only new ones)
    let mut doc_count = 0; let mut doc_skipped = 0;
    if let Ok(mut stmt) = conn.prepare("SELECT id, title, section_path, content FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name!='')") {
        let mut batch = Vec::new();
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            let doc_total = all.len();
            let mut doc_processed = 0;
            let report_every = if doc_total > 10 { doc_total / 5 } else { 3 };
            for row in all {
                if existing_doc_ids.contains(&row.0) { doc_skipped += 1; doc_processed += 1; continue; }
                let text = if !row.2.is_empty() { format!("{}: {} {}", row.1, row.2, row.3) } else { format!("{} {}", row.1, row.3) };
                let emb = embedder.embed(&text).unwrap_or_default();
                if !emb.is_empty() {
                    batch.push((row.0, emb));
                }
                if batch.len() >= 50 {
                    doc_count += crate::storage::vector::insert_vectors(conn, &doc_table, &batch.iter().map(|(id, v)| (*id, v.as_slice())).collect::<Vec<_>>())?;
                    batch.clear();
                }
                doc_processed += 1;
                if doc_processed % report_every == 0 {
                    eprint!("\r  Vectors: {}/{} docs ({}%)...", doc_processed, doc_total, doc_processed * 100 / doc_total.max(1));
                }
            }
            if doc_total > 0 {
                eprint!("\r  Vectors: {}/{} docs done.", doc_processed, doc_total);
                eprintln!();
            }
        }
        if !batch.is_empty() {
            doc_count += crate::storage::vector::insert_vectors(conn, &doc_table, &batch.iter().map(|(id, v)| (*id, v.as_slice())).collect::<Vec<_>>())?;
        }
    }

    if sym_total > 0 && (sym_count > 0 || doc_count > 0) {
        eprintln!("  Vectors: {} new symbols ({} skipped), {} new docs ({} skipped)",
            sym_count, sym_skipped, doc_count, doc_skipped);
    }
    Ok((sym_count, doc_count))
}

/// Index only doc vectors (symbol vectors are done inline during parsing).
/// Uses content_hash to detect changed docs; only re-embeds docs that are new or modified.
/// Returns number of doc vectors created/updated.
pub fn index_doc_vectors(
    conn: &rusqlite::Connection,
    repo: &str,
    embedder: &dyn Embedder,
) -> anyhow::Result<usize> {
    if !crate::storage::vector::try_load(conn) {
        return Ok(0);
    }
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));

    // Collect existing vec0 rowids for dedup
    let existing_doc_ids: std::collections::HashSet<i64> = {
        let mut s = std::collections::HashSet::new();
        if let Ok(mut stmt) = conn.prepare(&format!("SELECT rowid FROM {doc_table}")) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() {
                    s.insert(r);
                }
            }
        }
        s
    };

    // Collect (rowid → content_hash) from doc_nodes for change detection
    let mut doc_hashes: std::collections::HashMap<i64, String> = std::collections::HashMap::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, content_hash FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name!='')",
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1).unwrap_or_default()))
        }) {
            for r in rows.flatten() {
                doc_hashes.insert(r.0, r.1);
            }
        }
    }

    // Clean stale vec0 entries: rowids that no longer exist in doc_nodes
    let stale_count = existing_doc_ids.iter().filter(|id| !doc_hashes.contains_key(id)).count();
    if stale_count > 0 {
        // vec0 doesn't support DELETE WHERE, so we DROP and rebuild (fast since no data)
        // Better: just skip them, they won't match any real doc queries
        eprintln!("  Vectors: {} stale doc vectors (docs removed)", stale_count);
    }

    let mut doc_count = 0;
    let mut doc_skipped = 0;
    let mut doc_updated = 0;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, title, section_path, content FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name!='')",
    ) {
        let mut batch_new = Vec::new();    // New docs: INSERT OR IGNORE
        let mut batch_update = Vec::new(); // Changed docs: INSERT OR REPLACE
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            let doc_total = all.len();
            let mut doc_processed = 0;
            let report_every = if doc_total > 10 { doc_total / 5 } else { 3 };
            for row in all {
                let (id, title, section_path, content) = (row.0, &row.1, &row.2, &row.3);
                let hash_text = format!("{}:{}:{}", title, section_path, content);
                let new_hash = crate::storage::dedup::hash_content(&hash_text);
                let stored_hash = doc_hashes.get(&id).map(|s| s.as_str()).unwrap_or("");

                let has_vec = existing_doc_ids.contains(&id);
                let hash_changed = stored_hash.is_empty() || stored_hash != new_hash;

                if has_vec && !hash_changed {
                    // Unchanged – skip
                    doc_skipped += 1;
                } else {
                    // New or changed – embed and insert/replace
                    let text = format!("{}: {} {}", title, section_path, content);
                    let emb = embedder.embed(&text).unwrap_or_default();
                    if !emb.is_empty() {
                        if has_vec {
                            batch_update.push((id, emb));
                        } else {
                            batch_new.push((id, emb));
                        }
                    }
                }

                // Flush batches
                if batch_new.len() >= 50 {
                    doc_count += crate::storage::vector::insert_vectors(
                        conn, &doc_table,
                        &batch_new.iter().map(|(i, v)| (*i, v.as_slice())).collect::<Vec<_>>(),
                    )?;
                    batch_new.clear();
                }
                if batch_update.len() >= 50 {
                    doc_updated += replace_vectors(conn, &doc_table, &batch_update)?;
                    batch_update.clear();
                }

                doc_processed += 1;
                if doc_processed % report_every == 0 {
                    eprint!(
                        "\r  Vectors: {}/{} docs ({}%)...",
                        doc_processed, doc_total,
                        doc_processed * 100 / doc_total.max(1)
                    );
                }
            }
            if doc_total > 0 {
                eprint!("\r  Vectors: {}/{} docs done.", doc_processed, doc_total);
                eprintln!();
            }

            // Flush remaining
            if !batch_new.is_empty() {
                doc_count += crate::storage::vector::insert_vectors(
                    conn, &doc_table,
                    &batch_new.iter().map(|(i, v)| (*i, v.as_slice())).collect::<Vec<_>>(),
                )?;
            }
            if !batch_update.is_empty() {
                doc_updated += replace_vectors(conn, &doc_table, &batch_update)?;
            }
        }
    }

    if doc_count > 0 || doc_skipped > 0 || doc_updated > 0 {
        eprintln!(
            "  Vectors: {} new + {} updated docs ({} skipped)",
            doc_count, doc_updated, doc_skipped
        );
    }
    Ok(doc_count + doc_updated)
}

/// Replace vectors for given rowids (used when doc content changed).
/// vec0 INSERT OR REPLACE overwrites the existing embedding for the same rowid.
fn replace_vectors(conn: &Connection, table: &str, rows: &[(i64, Vec<f32>)]) -> anyhow::Result<usize> {
    let mut count = 0;
    for (rowid, vec) in rows {
        let json = format!(
            "[{}]",
            vec.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",")
        );
        conn.execute(
            &format!("INSERT OR REPLACE INTO {table} (rowid, embedding) VALUES (?1, ?2)"),
            rusqlite::params![rowid, json],
        )?;
        count += 1;
    }
    Ok(count)
}

// ── Embedder factory ──────────────────────────────────────────────────

/// Returns the best available embedder: CandleEmbedder if model is present, TextEmbedder otherwise.
pub fn get_embedder() -> Box<dyn Embedder + Send + Sync> {
    if CandleEmbedder::model_available() {
        Box::new(CandleEmbedder)
    } else {
        Box::new(TextEmbedder)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_same() {
        let v = vec![1.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_empty() {
        assert_eq!(cosine_similarity(&[], &[1.0]), 0.0);
    }

    #[test]
    fn test_text_embedder_returns_nonempty() {
        let e = TextEmbedder;
        let emb = e.embed("hello world").unwrap();
        assert!(!emb.is_empty());
    }

    #[test]
    fn test_text_embedder_similarity_same() {
        let e = TextEmbedder;
        let s = e.similarity(&[1.0, 2.0], &[1.0, 2.0]);
        assert!((s - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_model_available() {
        // models/bge-small-zh/ should exist from build.rs
        assert!(CandleEmbedder::model_available());
    }

    #[test]
    fn test_candle_embed_returns_384_dim() {
        let e = CandleEmbedder;
        let emb = e.embed("测试文本").unwrap();
        let d = emb.len();
        assert_eq!(d, 512, "embed dim {} not 512", d);
        // Check valid floats
        for &v in &emb {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn test_candle_similarity_same_text() {
        let e = CandleEmbedder;
        let emb1 = e.embed("相同的文本").unwrap();
        let emb2 = e.embed("相同的文本").unwrap();
        let sim = e.similarity(&emb1, &emb2);
        assert!(sim > 0.99, "same text similarity {} should be > 0.99", sim);
    }

    #[test]
    fn test_candle_similarity_different_text() {
        let e = CandleEmbedder;
        let emb1 = e.embed("计算机科学").unwrap();
        let emb2 = e.embed("美食烹饪").unwrap();
        let dim = emb2.len();
        // bge-small-zh should be 384-dim
        assert_eq!(dim, 512, "unexpected dim {}", dim);
        let sim = e.similarity(&emb1, &emb2);
        assert!(sim < 0.85, "different text similarity {} should be < 0.85", sim);
    }

    #[test]
    fn test_get_embedder_returns_candle() {
        let e = get_embedder();
        assert_eq!(e.dimension(), 384);
    }
}
