// build.rs — 编译 sqlite-vec（静态链接）+ 下载 bge-small-zh 嵌入模型

use std::path::Path;

const MODEL_BASE: &str = "https://modelscope.cn/models/BAAI/bge-small-zh/resolve/master";
const MODEL_FILES: &[&str] = &["pytorch_model.bin", "config.json", "tokenizer.json"];

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn main() {
    // ── 1. 静态编译 sqlite-vec ────────────────────────────────────
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    cc::Build::new()
        .files(&[
            manifest_dir.join("vendor/sqlite-vec/sqlite-vec.c"),
            manifest_dir.join("vendor/vec0_static.c"),
        ])
        .include(manifest_dir.join("vendor"))             // sqlite3.h
        .include(manifest_dir.join("vendor/sqlite-vec"))  // sqlite-vec.h
        .define("SQLITE_CORE", None)   // 使用 SQLite 核心 API，而非扩展 API
        .opt_level(2)
        .warnings(true)
        .compile("sqlite_vec0");

    println!("cargo:warning=sqlite-vec compiled and linked statically");

    // ── 2. 下载 bge-small-zh 模型 ─────────────────────────────────
    let out_dir = manifest_dir.join("models").join("bge-small-zh");
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
