// Embedding module: vector embeddings for semantic code-doc search
// Architecture: trait-based embedding interface, ONNX runtime backend (bge-small-zh ~96MB)
// TODO: Download bge-small-zh ONNX model and integrate ort crate for inference
use rusqlite::Connection;

/// Embedding vector (placeholder until ONNX model is integrated)
pub type Embedding = Vec<f32>;

/// Trait for embedding providers
pub trait Embedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding>;
    fn dimension(&self) -> usize;
}

/// Stub embedder (returns empty vec until ONNX model is downloaded)
pub struct StubEmbedder;
impl Embedder for StubEmbedder {
    fn embed(&self, _text: &str) -> anyhow::Result<Embedding> {
        Ok(Vec::new())
    }
    fn dimension(&self) -> usize { 384 } // bge-small-zh dimension
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

/// Link documents to symbols by embedding similarity.
/// When ONNX model is integrated, this will use real embeddings.
/// For now, stores placeholder entries with strength=0.0.
pub fn link_docs_to_symbols(conn: &Connection, _embedder: &dyn Embedder, repo: &str, _threshold: f64) -> anyhow::Result<usize> {
    // TODO: when real embedder is available:
    // 1. SELECT id, content FROM doc_nodes WHERE repo=?
    // 2. SELECT id, definition FROM symbols WHERE repo=?
    // 3. Embed both, compute cosine similarity
    // 4. INSERT OR REPLACE INTO doc_code_links WHERE similarity > threshold
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM doc_nodes WHERE repo=?1",
        rusqlite::params![repo],
        |r| r.get(0),
    )?;
    Ok(count as usize)
}

/// Generate a simple text-based "embedding" from a symbol definition for testing.
/// Concatenates name, kind, signature, and first 512 chars of definition.
/// This is NOT a real embedding — just a placeholder until ONNX is integrated.
pub fn symbol_text_for_embedding(conn: &Connection, symbol_id: i64) -> Option<String> {
    conn.query_row(
        "SELECT name || ' ' || COALESCE(signature,'') || ' ' || COALESCE(substr(definition,1,512),'') FROM symbols WHERE id=?1",
        rusqlite::params![symbol_id],
        |r| r.get(0),
    ).ok()
}
