// Embedding module — API-based via OpenAI-compatible /v1/embeddings.
use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use rusqlite::Connection;

pub type Embedding = Vec<f32>;

pub trait Embedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding>;
    fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<Embedding>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn batch_size(&self) -> usize { 64 }
    fn text_limit(&self) -> usize { 2000 }
    fn max_chars_per_batch(&self) -> usize { 90000 }
    fn dimension(&self) -> usize;
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32;
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let dot: f32 = a.iter().zip(b).map(|(x,y)| x*y).sum();
    let na: f32 = a.iter().map(|x| x*x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x*x).sum::<f32>().sqrt();
    if na < 1e-10 || nb < 1e-10 { return 0.0; }
    (dot / (na * nb)).max(0.0).min(1.0)
}

pub struct ApiEmbedder {
    client: reqwest::blocking::Client,
    api_base: String,
    api_key: Option<String>,
    model: String,
    batch_size: usize,
    text_limit: usize,
    max_chars_per_batch: usize,
    dimension: std::sync::Mutex<Option<usize>>,
}

impl ApiEmbedder {
    pub fn new(cfg: &crate::config::EmbeddingConfig) -> anyhow::Result<Self> {
        let mut builder = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30));
        if let Some(ref key) = cfg.api_key {
            let mut headers = reqwest::header::HeaderMap::new();
            let mut auth = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", key))
                .map_err(|e| anyhow::anyhow!("Invalid API key: {}", e))?;
            auth.set_sensitive(true);
            headers.insert(reqwest::header::AUTHORIZATION, auth);
            builder = builder.default_headers(headers);
        }
        Ok(Self {
            client: builder.build()?,
            api_base: cfg.api_base.trim_end_matches('/').to_string(),
            api_key: cfg.api_key.clone(),
            model: cfg.model.clone(),
            batch_size: cfg.batch_size,
            text_limit: cfg.text_limit,
            max_chars_per_batch: cfg.max_chars_per_batch,
            dimension: Mutex::new(cfg.dimension),
        })
    }

    fn call_api(&self, texts: &[String]) -> anyhow::Result<Vec<Embedding>> {
        let url = format!("{}/embeddings", self.api_base);
        let body = serde_json::json!({"model": self.model, "input": texts});
        let max_retries = 3;
        let mut last_err = None;
        for attempt in 0..max_retries {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                std::thread::sleep(delay);
            }
            let mut req = self.client.post(&url).json(&body);
            if let Some(ref key) = self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            let resp = match req.send() {
                Ok(r) => r,
                Err(e) => { last_err = Some(anyhow::anyhow!("HTTP send: {}", e)); continue; }
            };
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().unwrap_or_default();
                last_err = Some(anyhow::anyhow!("Embedding API {}: {}", status, text));
                continue;
            }
            let json: serde_json::Value = match resp.json() {
                Ok(j) => j,
                Err(e) => { last_err = Some(anyhow::anyhow!("JSON parse: {}", e)); continue; }
            };
            let data = match json["data"].as_array() {
                Some(d) => d,
                None => { last_err = Some(anyhow::anyhow!("No 'data' in response: {}", json)); continue; }
            };
            let mut results: Vec<(usize, Embedding)> = Vec::with_capacity(data.len());
            for item in data {
                let idx = item["index"].as_i64().unwrap_or(0) as usize;
                let vec: Vec<f32> = match item["embedding"].as_array() {
                    Some(arr) => arr.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect(),
                    None => continue,
                };
                results.push((idx, vec));
            }
            results.sort_by_key(|(i, _)| *i);
            return Ok(results.into_iter().map(|(_, v)| v).collect());
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All {} retries failed", max_retries)))
    }
}

impl Embedder for ApiEmbedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding> {
        let results = self.call_api(&[text.to_string()])?;
        let emb = results.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Empty embedding response"))?;
        let mut dim = self.dimension.lock().unwrap();
        if dim.is_none() { *dim = Some(emb.len()); }
        Ok(emb)
    }
    fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<Embedding>> {
        self.call_api(&texts.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
    fn batch_size(&self) -> usize { self.batch_size }
    fn text_limit(&self) -> usize { self.text_limit }
    fn max_chars_per_batch(&self) -> usize { self.max_chars_per_batch }
    fn dimension(&self) -> usize { self.dimension.lock().unwrap().unwrap_or(1024) }
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32 { cosine_similarity(a, b) }
}

// ── Vector indexing ───────────────────────────────────────────────────

pub fn index_vectors(conn: &Connection, repo: &str, embedder: &dyn Embedder) -> anyhow::Result<(usize, usize)> {
    if !crate::storage::vector::try_load(conn) { return Ok((0, 0)); }
    crate::storage::vector::create_tables(conn, repo, embedder.dimension())?;
    let sym_table = format!("symbol_vec_{}", repo.replace('-', "_"));
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));
    let batch_limit = embedder.batch_size();

    let existing_sym_ids: HashSet<i64> = {
        let mut s = HashSet::new();
        if let Ok(mut st) = conn.prepare(&format!("SELECT rowid FROM {sym_table}")) {
            if let Ok(rows) = st.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() { s.insert(r); }
            }
        }
        s
    };
    let existing_doc_ids: HashSet<i64> = {
        let mut s = HashSet::new();
        if let Ok(mut st) = conn.prepare(&format!("SELECT rowid FROM {doc_table}")) {
            if let Ok(rows) = st.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() { s.insert(r); }
            }
        }
        s
    };

    // Symbol vectors — smart batch by character count
    let mut sym_count = 0; let mut sym_skipped = 0; let mut sym_total = 0;
    if let Ok(mut stmt) = conn.prepare("SELECT id, name, kind, doc_comment FROM symbols WHERE repo=?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            sym_total = all.len();
            let mut processed = 0;
            let mut text_batch: Vec<(i64, String)> = Vec::new();
            let mut vec_batch: Vec<(i64, Vec<f32>)> = Vec::new();
            let mut batch_chars = 0usize;
            for row in all {
                if existing_sym_ids.contains(&row.0) { sym_skipped += 1; processed += 1; continue; }
                let text = if row.3.is_empty() {
                    format!("{} | {}", row.1, row.2)
                } else {
                    format!("{} | {} | {}", row.1, row.2, row.3)
                };
                let chars = text.len();
                batch_chars += chars;
                text_batch.push((row.0, text));
                processed += 1;
                if batch_chars >= embedder.max_chars_per_batch() || text_batch.len() >= embedder.batch_size() {
                    flush_symbol_batch(&texts_of(&text_batch), embedder, &mut vec_batch, &text_batch);
                    sym_count += insert_vec_batch(conn, &sym_table, &mut vec_batch)?;
                    text_batch.clear();
                    batch_chars = 0;
                }
                if processed % (sym_total/10).max(1) == 0 {
                    eprint!("\r  Vectors: {}/{} symbols ({}%)...", processed, sym_total, processed*100/sym_total);
                }
            }
            if !text_batch.is_empty() {
                flush_symbol_batch(&texts_of(&text_batch), embedder, &mut vec_batch, &text_batch);
                sym_count += insert_vec_batch(conn, &sym_table, &mut vec_batch)?;
            }
            if sym_total > 0 { eprint!("\r  Vectors: {}/{} symbols done.\n", processed, sym_total); }
        }
    }

    // Doc vectors — use embedder batch_size (API limit, typically 64)
    let mut doc_count = 0; let mut doc_skipped = 0;
    if let Ok(mut stmt) = conn.prepare("SELECT id, title, section_path, content FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name!='') AND content != ''") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            let mut text_batch: Vec<(i64, String)> = Vec::new();
            let mut vec_batch: Vec<(i64, Vec<f32>)> = Vec::new();
            let mut batch_chars = 0usize;
            for row in all {
                if existing_doc_ids.contains(&row.0) { doc_skipped += 1; continue; }
                let content: String = {
                    let c: String = row.3.chars().take(embedder.text_limit()).collect();
                    c.replace('\r', "")
                };
                let text = if !row.2.is_empty() { format!("{}: {} {}", row.1, row.2, content) } else { format!("{} {}", row.1, content) };
                let chars = text.len();
                batch_chars += chars;
                text_batch.push((row.0, text));
                if batch_chars >= embedder.max_chars_per_batch() || text_batch.len() >= embedder.batch_size() {
                    flush_symbol_batch(&texts_of(&text_batch), embedder, &mut vec_batch, &text_batch);
                    doc_count += insert_vec_batch(conn, &doc_table, &mut vec_batch)?;
                    text_batch.clear();
                    batch_chars = 0;
                }
            }
            if !text_batch.is_empty() {
                flush_symbol_batch(&texts_of(&text_batch), embedder, &mut vec_batch, &text_batch);
                doc_count += insert_vec_batch(conn, &doc_table, &mut vec_batch)?;
            }
        }
    }

    if sym_total > 0 || doc_count > 0 {
        eprintln!("  Vectors: {} symbols ({} skipped), {} docs ({} skipped)", sym_count, sym_skipped, doc_count, doc_skipped);
    }
    Ok((sym_count, doc_count))
}

pub fn index_doc_vectors(conn: &Connection, repo: &str, embedder: &dyn Embedder) -> anyhow::Result<usize> {
    if !crate::storage::vector::try_load(conn) { return Ok(0); }
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));
    let batch_limit = embedder.batch_size();

    let existing_doc_ids: HashSet<i64> = {
        let mut s = HashSet::new();
        if let Ok(mut st) = conn.prepare(&format!("SELECT rowid FROM {doc_table}")) {
            if let Ok(rows) = st.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() { s.insert(r); }
            }
        }
        s
    };

    let mut doc_count = 0;
    if let Ok(mut stmt) = conn.prepare("SELECT id, title, section_path, content FROM doc_nodes WHERE repo=?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            let mut text_batch: Vec<(i64, String)> = Vec::new();
            let mut vec_batch: Vec<(i64, Vec<f32>)> = Vec::new();
            for row in all {
                if existing_doc_ids.contains(&row.0) { continue; }
                let limit = embedder.text_limit();
                let content: String = if row.3.len() > limit { row.3.chars().take(limit).collect() } else { row.3.clone() };
                text_batch.push((row.0, format!("{} {} {}", row.1, row.2, content)));
                if text_batch.len() >= batch_limit {
                    let texts: Vec<&str> = text_batch.iter().map(|(_, t)| t.as_str()).collect();
                    match embedder.embed_batch(&texts) {
                        Ok(embs) => {
                            for ((id, _), emb) in text_batch.iter().zip(embs) {
                                if !emb.is_empty() { vec_batch.push((*id, emb)); }
                            }
                        }
                        Err(e) => eprintln!("  ⚠ embed batch failed ({} texts, sample: {:?}): {}", 
                            texts.len(), 
                            texts.first().map(|s| &s[..s.len().min(100)]).unwrap_or(""),
                            e),
                    }
                    text_batch.clear();
                    if !vec_batch.is_empty() {
                        doc_count += crate::storage::vector::insert_vectors(conn, &doc_table, &vec_batch.iter().map(|(i,v)| (*i, v.as_slice())).collect::<Vec<_>>())?;
                        vec_batch.clear();
                    }
                }
            }
            if !text_batch.is_empty() {
                let texts: Vec<&str> = text_batch.iter().map(|(_, t)| t.as_str()).collect();
                match embedder.embed_batch(&texts) {
                    Ok(embs) => {
                        for ((id, _), emb) in text_batch.iter().zip(embs) {
                            if !emb.is_empty() { vec_batch.push((*id, emb)); }
                        }
                    }
                    Err(e) => eprintln!("  ⚠ embed remainder batch failed: {}", e),
                }
            }
            if !vec_batch.is_empty() {
                doc_count += crate::storage::vector::insert_vectors(conn, &doc_table, &vec_batch.iter().map(|(i,v)| (*i, v.as_slice())).collect::<Vec<_>>())?;
            }
        }
    }
    Ok(doc_count)
}

// ── Smart batching helpers ─────────────────────────────────────────────

fn texts_of<'a>(batch: &'a [(i64, String)]) -> Vec<&'a str> {
    batch.iter().map(|(_, t)| t.as_str()).collect()
}

fn flush_symbol_batch(
    texts: &[&str],
    embedder: &dyn Embedder,
    vec_batch: &mut Vec<(i64, Vec<f32>)>,
    text_batch: &[(i64, String)],
) {
    match embedder.embed_batch(texts) {
        Ok(embs) => {
            for ((id, _), emb) in text_batch.iter().zip(embs) {
                if !emb.is_empty() { vec_batch.push((*id, emb)); }
            }
        }
        Err(e) => eprintln!("  ⚠ embed batch failed ({} texts, sample: {:?}): {}",
            texts.len(),
            texts.first().map(|s| &s[..s.len().min(100)]).unwrap_or(""),
            e),
    }
}

fn insert_vec_batch(
    conn: &Connection,
    table: &str,
    vec_batch: &mut Vec<(i64, Vec<f32>)>,
) -> anyhow::Result<usize> {
    if vec_batch.is_empty() { return Ok(0); }
    let count = crate::storage::vector::insert_vectors(
        conn, table,
        &vec_batch.iter().map(|(i, v)| (*i, v.as_slice())).collect::<Vec<_>>(),
    )?;
    vec_batch.clear();
    Ok(count)
}

/// Index file vectors — embed "file_path | summary" for each file
pub fn index_file_vectors(conn: &Connection, repo: &str, embedder: &dyn Embedder) -> anyhow::Result<usize> {
    if !crate::storage::vector::try_load(conn) { return Ok(0); }
    let file_table = format!("file_vec_{}", repo.replace('-', "_"));
    crate::storage::vector::create_tables(conn, repo, embedder.dimension())?;

    let existing_ids: HashSet<i64> = {
        let mut s = HashSet::new();
        if let Ok(mut st) = conn.prepare(&format!("SELECT rowid FROM {file_table}")) {
            if let Ok(rows) = st.query_map([], |r| r.get::<_, i64>(0)) {
                for r in rows.flatten() { s.insert(r); }
            }
        }
        s
    };

    let mut file_count = 0;
    if let Ok(mut stmt) = conn.prepare("SELECT id, file_path, summary FROM files WHERE repo=?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        }) {
            let all: Vec<_> = rows.flatten().collect();
            let mut text_batch: Vec<(i64, String)> = Vec::new();
            let mut vec_batch: Vec<(i64, Vec<f32>)> = Vec::new();
            for row in all {
                if existing_ids.contains(&row.0) { continue; }
                let text = if row.2.is_empty() {
                    row.1.clone()
                } else {
                    format!("{} | {}", row.1, row.2)
                };
                text_batch.push((row.0, text));
                if text_batch.len() >= embedder.batch_size() {
                    flush_symbol_batch(&texts_of(&text_batch), embedder, &mut vec_batch, &text_batch);
                    file_count += insert_vec_batch(conn, &file_table, &mut vec_batch)?;
                    text_batch.clear();
                }
            }
            if !text_batch.is_empty() {
                flush_symbol_batch(&texts_of(&text_batch), embedder, &mut vec_batch, &text_batch);
                file_count += insert_vec_batch(conn, &file_table, &mut vec_batch)?;
            }
        }
    }
    Ok(file_count)
}

// ── Factory ───────────────────────────────────────────────────────────

pub fn get_embedder() -> anyhow::Result<Box<dyn Embedder + Send + Sync>> {
    let config = crate::config::Config::load().unwrap_or_default();
    match &config.embedding {
        Some(cfg) => {
            let embedder = ApiEmbedder::new(cfg)?;
            embedder.embed("init")?; // auto-detect dimension
            Ok(Box::new(embedder))
        }
        None => anyhow::bail!("No embedding config. Add to ~/.codeloom/config.yaml:\n  embedding:\n    api_base: \"http://host:port/v1\"\n    model: \"bge-m3\""),
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
        assert!((cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_empty() {
        assert_eq!(cosine_similarity(&[], &[1.0]), 0.0);
    }
}
