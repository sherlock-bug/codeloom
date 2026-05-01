use tree_sitter::Parser;
use walkdir::WalkDir;
use crate::storage::symbols::Symbol;

pub struct FileInfo { pub path: String, pub language: &'static str, pub modified: std::time::SystemTime }

pub fn create_parser(language: &str) -> Option<Parser> {
    let mut p = Parser::new();
    let lang = match language {
        "cpp" => tree_sitter_cpp::LANGUAGE.into(), "python" => tree_sitter_python::LANGUAGE.into(),
        "java" => tree_sitter_java::LANGUAGE.into(), "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        "go" => tree_sitter_go::LANGUAGE.into(), _ => return None,
    };
    p.set_language(&lang).ok()?; Some(p)
}

pub fn detect_language(fp: &str) -> Option<&'static str> {
    match std::path::Path::new(fp).extension()?.to_str()? {
        "cpp"|"cc"|"cxx"|"hpp"|"hxx"|"c"|"h" => Some("cpp"),
        "py" => Some("python"), "java" => Some("java"),
        "ts"|"tsx"|"js"|"jsx" => Some("typescript"), "go" => Some("go"),
        _ => None,
    }
}

pub fn collect_files(root: &str) -> Vec<FileInfo> {
    let git_files = get_git(root);
    let mut files = Vec::new(); let mut skipped = 0usize;
    for e in WalkDir::new(root).into_iter().filter_map(|r| r.ok()).filter(|e| e.file_type().is_file()) {
        let ps = e.path().to_string_lossy().to_string();
        if ps.contains("/.git/") { continue; }
        if let Some(ref gf) = git_files { if !gf.contains(&ps) && !gf.iter().any(|g| ps.ends_with(g)) { skipped+=1; continue; } }
        if let Some(lang) = detect_language(&ps) {
            if let Ok(meta) = e.metadata() { if let Ok(modified) = meta.modified() {
                files.push(FileInfo { path: ps, language: lang, modified });
            }}
        }
    }
    if skipped>0 { println!("  Skipped {} gitignored files", skipped); }
    files
}

fn get_git(root: &str) -> Option<std::collections::HashSet<String>> {
    let o = std::process::Command::new("git").args(["ls-files","--cached","--others","--exclude-standard"]).current_dir(root).output().ok()?;
    if !o.status.success() { return None; }
    let s: std::collections::HashSet<_> = String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    if s.is_empty() { None } else { Some(s) }
}

pub fn parse_file(file: &FileInfo, parser: &mut Parser, repo: &str) -> anyhow::Result<(Vec<Symbol>, Vec<(usize,usize,String)>)> {
    let source = std::fs::read_to_string(&file.path)?;
    let tree = parser.parse(&source, None).ok_or_else(|| anyhow::anyhow!("parse: {}", file.path))?;
    let mut syms = Vec::new(); let mut edges: Vec<(usize,usize,String)> = Vec::new();
    match file.language {
        "cpp" => crate::indexer::queries::cpp::extract(&source, tree.root_node(), file, repo, &mut syms, &mut edges),
        _ => {}
    }
    Ok((syms, edges))
}

pub fn extract_text(source: &str, start: u32, end: u32) -> String {
    source.lines().skip(start as usize - 1).take((end - start + 1) as usize).collect::<Vec<_>>().join("\n")
}
