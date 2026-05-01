// ONNX embedding with bge-small-zh — model download infrastructure
// First use triggers download via wget/curl from HuggingFace
// TODO: Wire candle-onnx inference. For now, delegates to TextEmbedder.
use std::path::PathBuf;
use std::process::Command;

use super::{Embedder, Embedding, TextEmbedder};

const MODEL_URL: &str = "https://huggingface.co/BAAI/bge-small-zh/resolve/main";

pub struct OnnxEmbedder {
    inner: TextEmbedder,
}

impl OnnxEmbedder {
    pub fn new() -> anyhow::Result<Self> {
        let _model_dir = ensure_model()?;
        Ok(Self { inner: TextEmbedder })
    }
}

impl Embedder for OnnxEmbedder {
    fn embed(&self, text: &str) -> anyhow::Result<Embedding> { self.inner.embed(text) }
    fn dimension(&self) -> usize { 384 }
    fn similarity(&self, a: &[f32], b: &[f32]) -> f32 { self.inner.similarity(a, b) }
}

/// Download bge-small-zh model files (~96MB) on first use
pub fn ensure_model() -> anyhow::Result<PathBuf> {
    let model_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codeloom/models/bge-small-zh");

    for file in &["model.onnx", "vocab.txt"] {
        let dest = model_dir.join(file);
        if !dest.exists() {
            std::fs::create_dir_all(&model_dir)?;
            let url = format!("{}/{}", MODEL_URL, file);
            let size = if *file == "model.onnx" { "96MB" } else { "100KB" };
            eprintln!("Downloading {} (~{})...", file, size);

            let status = Command::new("wget")
                .args(["-q", "-O", dest.to_str().unwrap_or("model.onnx"), &url])
                .status()
                .or_else(|_| {
                    Command::new("curl")
                        .args(["-sL", "-o", dest.to_str().unwrap_or("model.onnx"), &url])
                        .status()
                })?;

            if !status.success() {
                eprintln!("Failed to download {}. Please manually download from:", file);
                eprintln!("  {}", url);
                eprintln!("  and save to {:?}", dest);
                anyhow::bail!("Download failed for {}", file);
            }
        }
    }
    Ok(model_dir)
}
