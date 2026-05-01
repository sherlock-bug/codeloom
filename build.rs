fn main() {
    // 编译时将 bge-small-zh ONNX 模型所在目录告知 cargo
    // 模型文件通过 include_bytes! 在 embedding/mod.rs 中嵌入
    println!("cargo:rerun-if-changed=src/embedding/model.onnx");
}
