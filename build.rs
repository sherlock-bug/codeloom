// build.rs — 编译 sqlite-vec（静态链接）
use std::path::Path;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    cc::Build::new()
        .files(&[
            manifest_dir.join("vendor/sqlite-vec/sqlite-vec.c"),
            manifest_dir.join("vendor/vec0_static.c"),
        ])
        .include(manifest_dir.join("vendor"))
        .include(manifest_dir.join("vendor/sqlite-vec"))
        .define("SQLITE_CORE", None)
        .opt_level(2)
        .warnings(true)
        .compile("sqlite_vec0");
    println!("cargo:warning=sqlite-vec compiled and linked statically");
}
