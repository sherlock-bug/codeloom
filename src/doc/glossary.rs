use regex::Regex;
#[derive(Debug)]
pub struct GlossaryEntry { pub alias: String, pub branch_name: String, pub description: String }
pub fn parse_branch_glossary(content: &str) -> Vec<GlossaryEntry> {
    let mut entries=Vec::new();
    let re=Regex::new(r"^#{1,4}\s+(.+?)\s*[\(/→]\s*([a-zA-Z0-9][-a-zA-Z0-9/_\.]+)\s*[\)]?").unwrap();
    let lines:Vec<&str>=content.lines().collect();
    for i in 0..lines.len() {
        if let Some(caps)=re.captures(lines[i]) {
            let alias=caps[1].trim().to_string();
            let branch=caps[2].trim().to_string();
            let mut desc=Vec::new();
            for j in i+1..lines.len() { let l=lines[j].trim(); if l.is_empty()||l.starts_with('#') { break; } desc.push(l); }
            entries.push(GlossaryEntry{alias,branch_name:branch,description:desc.join(" ")});
        }
    }
    entries
}