// Embedding module: vector embeddings + text-based similarity for code-doc linking
// When ONNX model (bge-small-zh) is integrated, swap TextEmbedder for OrtEmbedder.
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
pub mod onnx;

/// Embedding vector (768-dim for bge-small-zh, 384-dim placeholder)
pub type Embedding = Vec<f32>;

/// Trait for embedding providers — swap TextEmbedder → OrtEmbedder when ONNX is ready
pub trait Embedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding>;
    fn dimension(&self) -> usize;
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32;
}

/// Cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let dot: f32 = a.iter().zip(b).map(|(x,y)| x*y).sum();
    let na: f32 = a.iter().map(|x| x*x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x*x).sum::<f32>().sqrt();
    if na < 1e-10 || nb < 1e-10 { return 0.0; }
    (dot / (na * nb)).max(0.0).min(1.0)
}

// ── Text-based embedder (Jaccard token overlap) ──────────────────────

/// TextEmbedder uses Jaccard similarity on token sets.
/// Fast, zero-dependency, works well for code symbol matching.
/// Swap for OrtEmbedder when ONNX model is downloaded.
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

/// Stub embedder (placeholder until ONNX model is downloaded)
pub struct StubEmbedder;
impl Embedder for StubEmbedder {
    fn embed(&self, _text: &str) -> anyhow::Result<Embedding> { Ok(Vec::new()) }
    fn dimension(&self) -> usize { 384 }
    fn similarity(&self, _a: &[f32], _b: &[f32]) -> f32 { 0.0 }
}

// ── Tokenization ─────────────────────────────────────────────────────

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

// ── Code-Doc Linking ─────────────────────────────────────────────────

/// Link documents to symbols by text similarity with token-based pre-filtering.
/// Only pairs sharing at least one token are compared (inverted index optimization).
/// Stores doc_code_links entries when Jaccard similarity > threshold.
pub fn link_docs_to_symbols(
    conn: &Connection,
    embedder: &dyn Embedder,
    repo: &str,
    threshold: f64,
) -> anyhow::Result<usize> {
    // Get all doc_nodes
    let mut doc_stmt = conn.prepare(
        "SELECT id, title, section_path, content FROM doc_nodes WHERE repo=?1"
    )?;
    let docs: Vec<(i64, String, String, String)> = doc_stmt.query_map(
        rusqlite::params![repo],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    )?.filter_map(|r| r.ok()).collect();

    if docs.is_empty() { return Ok(0); }

    // Get all symbols with definitions
    let mut sym_stmt = conn.prepare(
        "SELECT id, name, kind, definition FROM symbols WHERE repo=?1"
    )?;
    let symbols: Vec<(i64, String, String, String)> = sym_stmt.query_map(
        rusqlite::params![repo],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    )?.filter_map(|r| r.ok()).collect();

    // Tokenize all docs
    let doc_tokens: Vec<(i64, HashSet<u64>)> = docs.iter()
        .map(|(id, title, section, content)| {
            let text = if !section.is_empty() {
                format!("{}: {}\n{}", title, section, content)
            } else {
                format!("{}\n{}", title, content)
            };
            let tokens: HashSet<u64> = embedder.embed(&text).unwrap_or_default()
                .into_iter().map(|x| x as u64).collect();
            (*id, tokens)
        })
        .collect();

    // Tokenize all symbols
    let sym_tokens: Vec<(i64, HashSet<u64>)> = symbols.iter()
        .map(|(id, name, kind, def)| {
            let text = format!("{} {} {}", kind, name, def);
            let tokens: HashSet<u64> = embedder.embed(&text).unwrap_or_default()
                .into_iter().map(|x| x as u64).collect();
            (*id, tokens)
        })
        .collect();

    // Build inverted index: token → symbol indices
    let mut token_index: HashMap<u64, Vec<usize>> = HashMap::new();
    for (si, (_, tokens)) in sym_tokens.iter().enumerate() {
        for token in tokens {
            token_index.entry(*token).or_default().push(si);
        }
    }

    // Clear existing links for this repo
    conn.execute(
        "DELETE FROM doc_code_links WHERE doc_node_id IN (SELECT id FROM doc_nodes WHERE repo=?1)",
        rusqlite::params![repo],
    )?;

    let mut ins = conn.prepare(
        "INSERT OR REPLACE INTO doc_code_links (doc_node_id, symbol_id, link_type, strength, source) VALUES (?1, ?2, 'semantic', ?3, 'text')"
    )?;

    let mut count = 0;
    // For each doc, find candidate symbols via shared tokens
    for (doc_id, doc_tok) in &doc_tokens {
        let mut candidates: HashSet<usize> = HashSet::new();
        for token in doc_tok {
            if let Some(sym_indices) = token_index.get(token) {
                candidates.extend(sym_indices);
            }
        }
        // Only compute Jaccard for candidates
        for &si in &candidates {
            let (sym_id, sym_tok) = &sym_tokens[si];
            let intersection = doc_tok.intersection(sym_tok).count();
            let union = doc_tok.union(sym_tok).count();
            if union == 0 { continue; }
            let jaccard = intersection as f64 / union as f64;
            if jaccard >= threshold {
                ins.execute(rusqlite::params![doc_id, sym_id, jaccard])?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Symbol text for embedding (used by external tools)
pub fn symbol_text_for_embedding(conn: &Connection, symbol_id: i64) -> Option<String> {
    conn.query_row(
        "SELECT name || ' ' || COALESCE(signature,'') || ' ' || COALESCE(substr(definition,1,512),'') FROM symbols WHERE id=?1",
        rusqlite::params![symbol_id],
        |r| r.get(0),
    ).ok()
}
