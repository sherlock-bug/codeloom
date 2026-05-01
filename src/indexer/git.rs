use std::process::Command;
pub fn head_commit(repo_root: &str) -> Option<String> {
    let out = Command::new("git").args(["rev-parse","HEAD"]).current_dir(repo_root).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
pub fn current_branch(repo_root: &str) -> Option<String> {
    let out = Command::new("git").args(["rev-parse","--abbrev-ref","HEAD"]).current_dir(repo_root).output().ok()?;
    out.status.success().then(|| {
        let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if b == "HEAD" { "detached".into() } else { b }
    })
}
pub fn changed_files(repo_root: &str, from: &str, to: &str) -> Vec<String> {
    let out = Command::new("git").args(["diff","--name-only",from,to]).current_dir(repo_root).output().ok();
    out.and_then(|o| o.status.success().then(|| String::from_utf8_lossy(&o.stdout).lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty() && std::path::Path::new(s).exists()).collect())).unwrap_or_default()
}
pub fn merge_base(repo_root: &str, a: &str, b: &str) -> Option<String> {
    let out = Command::new("git").args(["merge-base",a,b]).current_dir(repo_root).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
pub fn is_ancestor(repo_root: &str, ancestor: &str, descendant: &str) -> bool {
    Command::new("git").args(["merge-base","--is-ancestor",ancestor,descendant]).current_dir(repo_root).output().map(|o| o.status.success()).unwrap_or(false)
}
