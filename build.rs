// build.rs — 编译时从 modelscope.cn 下载 bge-small-zh 嵌入模型到 models/ 目录
// 下载失败不阻塞编译，运行时自动降级到 TextEmbedder (Jaccard)

use std::path::Path;

const MODEL_BASE: &str = "https://modelscope.cn/models/BAAI/bge-small-zh/resolve/master";
const MODEL_FILES: &[&str] = &["pytorch_model.bin", "config.json", "tokenizer.json"];

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}



// ── sqlite-vec extension ───────────────────────────────────────────────

const VEC0_VERSION: &str = "v0.1.9";
const VEC0_URL: &str = "https://github.com/asg017/sqlite-vec/releases/download";

fn download_vec0() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("models")
        .join("sqlite-vec");
    let marker = out_dir.join("vec0.so");

    if marker.exists() {
        let sz = file_size(&marker);
        if sz > 50_000 {
            println!("cargo:warning=sqlite-vec vec0.so ready ({} KB)", sz / 1024);
            return;
        }
        let _ = std::fs::remove_file(&marker);
    }

    println!("cargo:warning=Downloading sqlite-vec {} extension...", VEC0_VERSION);
    std::fs::create_dir_all(&out_dir).ok();

    let arch = if cfg!(target_arch = "x86_64") { "x86_64" } 
               else if cfg!(target_arch = "aarch64") { "arm64" } 
               else { "x86_64" }; // fallback
    let url = format!(
        "{}/{}/sqlite-vec-{}-loadable-linux-{}.tar.gz",
        VEC0_URL, VEC0_VERSION, VEC0_VERSION, arch
    );
    
    // Download and extract
    let tarball = out_dir.join("vec0.tar.gz");
    let status = std::process::Command::new("curl")
        .args(["-sSL", "--connect-timeout", "10", "--max-time", "60", "-o"])
        .arg(&tarball).arg(&url).status();
    match status {
        Ok(s) if s.success() => {
            let _ = std::process::Command::new("tar")
                .args(["-xzf"]).arg(&tarball).arg("-C").arg(&out_dir).status();
            let _ = std::fs::remove_file(&tarball);
            if marker.exists() {
                println!("cargo:warning=sqlite-vec vec0.so ready");
            } else {
                println!("cargo:warning=sqlite-vec extraction failed, vector search will use fallback");
            }
        }
        _ => {
            let _ = std::fs::remove_file(&tarball);
            println!("cargo:warning=sqlite-vec download failed, vector search will use fallback");
        }
    }
}

fn main() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("models").join("bge-small-zh");
    let marker = out_dir.join("pytorch_model.bin");

    let existing = file_size(&marker);
    if existing > 50_000_000 {
        println!("cargo:warning=models/bge-small-zh/ ready ({} MB)", existing / 1_048_576);
        return;
    }
    if existing > 0 {
        let _ = std::fs::remove_file(&marker);
    }

    println!("cargo:warning=Downloading bge-small-zh model from modelscope.cn (~91MB, one-time)...");
    std::fs::create_dir_all(&out_dir).ok();

    let mut success = true;
    for &file in MODEL_FILES {
        let url = format!("{}/{}", MODEL_BASE, file);
        let dest = out_dir.join(file);
        if !try_download(file, &url, &dest) {
            success = false;
            break;
        }
    }

    if !success {
        let _ = std::fs::remove_dir_all(&out_dir);
        println!("cargo:warning=Model download incomplete - semantic search will use Jaccard fallback.");
        return;
    }

    download_vec0();
    let size = file_size(&marker);
    if size < 50_000_000 {
        println!("cargo:warning=pytorch_model.bin too small ({} bytes), will use Jaccard fallback", size);
        let _ = std::fs::remove_dir_all(&out_dir);
        return;
    }

    println!("cargo:warning=bge-small-zh model ready ({} MB)", size / 1_048_576);
}

fn try_download(name: &str, url: &str, dest: &Path) -> bool {
    println!("cargo:warning=  Downloading {}...", name);
    let status = std::process::Command::new("curl")
        .args(["-sL", "--connect-timeout", "10", "--max-time", "600", "-o"])
        .arg(dest)
        .arg(url)
        .status();
    match status {
        Ok(s) if s.success() => {
            let size = file_size(dest);
            println!("cargo:warning=  OK {} ({} bytes)", name, size);
            size > 0
        }
        _ => {
            let _ = std::fs::remove_file(dest);
            println!("cargo:warning=  x {} (download failed)", name);
            false
        }
    }
}
