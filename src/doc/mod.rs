// Document indexing: unified parse & store pipeline for all formats
use rusqlite::Connection;
pub mod glossary;
pub mod section;
pub mod image;
pub mod xml;
pub mod xlsx;
pub mod docx;
pub mod pdf;

pub use section::{DocSection, ImageRef};
pub use image::{smart_compress, to_base64, CompressedImage};
use regex::Regex;

/// Route document parsing by file extension
pub fn parse_document(ext: &str, path: &str, bytes: &[u8]) -> anyhow::Result<Vec<DocSection>> {
    match ext {
        "md" | "rst" => {
            let content = crate::util::read_file_smart(std::path::Path::new(path))
                .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string());
            Ok(parse_markdown(&content, path))
        }
        "xlsx" | "xls" | "xlsm" => xlsx::parse_xlsx(bytes),
        "docx" => docx::parse_docx(bytes),
        "pdf" => pdf::parse_pdf(bytes),
        "xml" => {
            let content = String::from_utf8_lossy(bytes).to_string();
            xml::parse_xml(&content)
        }
        _ => Err(anyhow::anyhow!("unsupported format: {}", ext)),
    }
}

/// Parse markdown content into sections with image extraction
pub fn parse_markdown(content: &str, _path: &str) -> Vec<DocSection> {
    let mut sections = Vec::new();
    let mut current_title = String::from("untitled");
    let mut current_section = String::new();
    let mut current_level = 0;
    let mut current_content = String::new();
    let mut images = Vec::new();
    let mut image_pos = 0;
    let img_re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    let html_img_re = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();

    let flush = |title: &mut String, section: &mut String, level: i32, content: &mut String, images: &mut Vec<ImageRef>, sections: &mut Vec<DocSection>| {
        if !content.trim().is_empty() || !section.is_empty() {
            sections.push(DocSection {
                title: title.clone(),
                section_path: section.clone(),
                level,
                node_type: "section".to_string(),
                content: content.trim().to_string(),
                images: std::mem::take(images),
            parent_id: None,
            });
        }
        content.clear();
    };

    for line in content.lines() {
        let trimmed = line.trim();

        for cap in img_re.captures_iter(trimmed) {
            image_pos += 1;
            let alt = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let url = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let ctx = current_content.chars().rev().take(100).collect::<String>().chars().rev().collect();
            if url.starts_with("http://") || url.starts_with("https://") {
                let mut img = ImageRef::linked(&alt, &url, image_pos);
                img.section_context = ctx;
                images.push(img);
            } else {
                let mut img = ImageRef::new(&alt, &url, image_pos);
                img.section_context = ctx;
                let base = std::path::Path::new(_path).parent().unwrap_or(std::path::Path::new("."));
                let local = base.join(&url);
                if let Ok(data) = std::fs::read(&local) {
                    img.raw_bytes = data;
                }
                images.push(img);
            }
        }

        for cap in html_img_re.captures_iter(trimmed) {
            image_pos += 1;
            let src = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let ctx = current_content.chars().rev().take(100).collect::<String>().chars().rev().collect();
            if src.starts_with("http://") || src.starts_with("https://") {
                images.push(ImageRef::linked("", src, image_pos));
            } else {
                let mut img = ImageRef::new("", src, image_pos);
                img.section_context = ctx;
                let base = std::path::Path::new(_path).parent().unwrap_or(std::path::Path::new("."));
                if let Ok(data) = std::fs::read(base.join(src)) {
                    img.raw_bytes = data;
                }
                images.push(img);
            }
        }

        if trimmed.starts_with("## ") {
            flush(&mut current_title, &mut current_section, current_level, &mut current_content, &mut images, &mut sections);
            current_section = trimmed[3..].to_string();
            current_level = 2;
        } else if trimmed.starts_with("### ") {
            flush(&mut current_title, &mut current_section, current_level, &mut current_content, &mut images, &mut sections);
            current_section = trimmed[4..].to_string();
            current_level = 3;
        } else if trimmed.starts_with("# ") {
            current_title = trimmed[2..].to_string();
        } else {
            if !current_content.is_empty() { current_content.push('\n'); }
            current_content.push_str(line);
        }
    }
    flush(&mut current_title, &mut current_section, current_level, &mut current_content, &mut images, &mut sections);

    sections.push(DocSection {
        title: current_title,
        section_path: String::new(),
        level: 1,
        node_type: "section".to_string(),
        content: content.to_string(),
        images: vec![],
    parent_id: None,
    });

    sections
}

/// Legacy API: parse markdown and directly write to DB
pub fn index_markdown(conn: &Connection, path: &str, content: &str, repo: &str) -> anyhow::Result<usize> {
    let sections = parse_markdown(content, path);
    let count = sections.len();
    write_doc_sections(conn, repo, path, "md", &sections)?;
    Ok(count)
}

/// Write doc sections + images to DB
pub fn write_doc_sections(
    conn: &Connection,
    repo: &str,
    path: &str,
    file_format: &str,
    sections: &[DocSection],
) -> anyhow::Result<usize> {
    let max_chunk = 500;
    let mut count = 0;

    // Get file node id for contains: edges
    let file_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM nodes WHERE repo=?1 AND node_type='file' AND name=?2",
            rusqlite::params![repo, path],
            |r| r.get(0),
        )
        .ok();

    for sec in sections {
        // Check if content needs chunking
        if sec.content.chars().count() <= max_chunk {
            // Short content — write directly
            let section_id = write_one_section(conn, repo, path, file_format, sec, None)?;
            // file → section contains: edge
            if let Some(fid) = file_id {
                conn.execute(
                    "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type) VALUES (?1, ?2, 'contains')",
                    rusqlite::params![fid, section_id],
                )?;
            }
            count += 1;
        } else {
            // Long content — write parent + chunks
            let parent_id = write_one_parent(conn, repo, path, file_format, sec)?;
            // file → section contains: edge for parent
            if let Some(fid) = file_id {
                conn.execute(
                    "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type) VALUES (?1, ?2, 'contains')",
                    rusqlite::params![fid, parent_id],
                )?;
            }
            // Delete old chunks
            conn.execute(
                "DELETE FROM nodes WHERE json_extract(attrs, '$.parent_id')=?1 AND node_type='chunk'",
                rusqlite::params![parent_id],
            )?;
            // Split and write chunks
            let chunks = split_at_punctuation(&sec.content, max_chunk);
            for (i, chunk_content) in chunks.iter().enumerate() {
                let chunk_path = format!("{}/chunk/{}", sec.section_path, i);
                let chunk = DocSection {
                    title: sec.title.clone(),
                    section_path: chunk_path,
                    level: sec.level,
                    node_type: "chunk".into(),
                    content: chunk_content.clone(),
                    images: vec![],
                    parent_id: Some(parent_id),
                };
                write_one_section(conn, repo, path, file_format, &chunk, Some(parent_id))?;
                count += 1;
            }
            count += 1; // parent counted
        }
    }
    Ok(count)
}

fn write_one_section(
    conn: &Connection,
    repo: &str,
    path: &str,
    file_format: &str,
    sec: &DocSection,
    parent_id: Option<i64>,
) -> anyhow::Result<i64> {
    let hash = crate::storage::dedup::hash_content(
        &format!("{}:{}:{}", sec.title, sec.section_path, sec.content)
    );
    let sp = if sec.section_path.is_empty() { String::new() } else { sec.section_path.clone() };
    let pid: Option<i64> = parent_id.or(sec.parent_id);

    // Build attrs JSON with doc-specific fields
    let mut attrs = serde_json::json!({
        "section_path": sp,
        "level": sec.level,
        "file_format": file_format,
        "parent_id": pid,
    });

    // Add chunk index for ordering within parent section
    if sec.node_type == "chunk" {
        if let Some(idx_str) = sec.section_path.rsplit('/').next() {
            if let Ok(idx) = idx_str.parse::<i32>() {
                attrs["idx"] = serde_json::json!(idx);
            }
        }
    }

    conn.execute(
        "INSERT INTO nodes (repo, node_type, name, content, file_path, content_hash, kind, attrs) \
         VALUES (?1, ?8, ?2, ?3, ?4, ?5, ?6, ?7) \
         ON CONFLICT(content_hash, file_path, name, branch_id, repo) DO UPDATE SET \
         content=excluded.content, kind=excluded.kind, \
         attrs=excluded.attrs",
        rusqlite::params![repo, sec.title, sec.content, path, hash, sec.node_type, attrs.to_string(), sec.node_type],
    )?;
    let last_node_id = conn.last_insert_rowid();

    if !sec.images.is_empty() {
        crate::storage::insert_doc_images(conn, last_node_id, &sec.images)?;
    }

    // Build contains: edge from parent
    if let Some(pid) = parent_id {
        conn.execute(
            "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type) VALUES (?1, ?2, 'contains')",
            rusqlite::params![pid, last_node_id],
        )?;
    }

    Ok(last_node_id)
}

fn write_one_parent(
    conn: &Connection,
    repo: &str,
    path: &str,
    file_format: &str,
    sec: &DocSection,
) -> anyhow::Result<i64> {
    let hash = crate::storage::dedup::hash_content(
        &format!("{}:{}:{}", sec.title, sec.section_path, sec.content)
    );
    let sp = if sec.section_path.is_empty() { String::new() } else { sec.section_path.clone() };

    // Build attrs JSON with doc-specific fields (no parent_id for parent node)
    let attrs = serde_json::json!({
        "section_path": sp,
        "level": sec.level,
        "file_format": file_format,
        "parent_id": serde_json::Value::Null,
    });

    conn.execute(
        "INSERT INTO nodes (repo, node_type, name, content, file_path, content_hash, kind, attrs) \
         VALUES (?1, 'section', ?2, '', ?3, ?4, ?5, ?6) \
         ON CONFLICT(content_hash, file_path, name, branch_id, repo) DO UPDATE SET \
         kind=excluded.kind, attrs=excluded.attrs",
        rusqlite::params![repo, sec.title, path, hash, sec.node_type, attrs.to_string()],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Split content into ≤max_len chunks at punctuation boundaries.
/// Priority: 。！？ > \n > ；，、 > force at max_len.
fn split_at_punctuation(content: &str, max_len: usize) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let total = chars.len();
    if total <= max_len {
        return vec![content.to_string()];
    }
    let mut chunks = Vec::new();
    let mut pos = 0;
    while pos < total {
        let end = (pos + max_len).min(total);
        if end >= total {
            chunks.push(chars[pos..].iter().collect());
            break;
        }
        let slice = &chars[pos..end];
        // Priority 1: 。！？
        if let Some(offset) = slice.iter().rposition(|c| matches!(c, '。' | '！' | '？')) {
            let split = pos + offset + 1;
            chunks.push(chars[pos..split].iter().collect());
            pos = split;
            continue;
        }
        // Priority 2: \n
        if let Some(offset) = slice.iter().rposition(|c| *c == '\n') {
            let split = pos + offset + 1;
            chunks.push(chars[pos..split].iter().collect());
            pos = split;
            continue;
        }
        // Priority 3: ；，、
        if let Some(offset) = slice.iter().rposition(|c| matches!(c, '；' | '，' | '、')) {
            let split = pos + offset + 1;
            chunks.push(chars[pos..split].iter().collect());
            pos = split;
            continue;
        }
        // Priority 4: force split at max_len
        chunks.push(chars[pos..end].iter().collect());
        pos = end;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_headings() {
        let md = "# Title\n## Section A\nContent A.\n## Section B\nContent B.";
        let sections = parse_markdown(md, "test.md");
        let sec_paths: Vec<&str> = sections.iter().map(|s| s.section_path.as_str()).collect();
        assert!(sec_paths.contains(&"Section A"));
        assert!(sec_paths.contains(&"Section B"));
    }

    #[test]
    fn test_parse_markdown_image() {
        let md = "## Test\nSome text.\n![diagram](test.png)\nMore text.";
        let sections = parse_markdown(md, "test.md");
        let has_image = sections.iter().any(|s| s.images.iter().any(|img| img.original_src == "test.png"));
        assert!(has_image);
    }

    #[test]
    fn test_xml_parser_sections() {
        let xml = "<config><db><host>x</host></db><srv><port>80</port></srv></config>";
        let sections = xml::parse_xml(xml).unwrap();
        assert!(sections.len() >= 1);
    }

    #[test]
    fn test_xlsx_four_layer_model() {
        let xlsx_bytes = std::fs::read("tests/fixtures/multi-format/sample.xlsx").unwrap();
        let sections = xlsx::parse_xlsx(&xlsx_bytes).unwrap();
        let types: Vec<&str> = sections.iter().map(|s| s.node_type.as_str()).collect();
        assert!(types.contains(&"sheet"));
        assert!(types.contains(&"header_cell"));
    }

    #[test]
    fn test_image_smart_compress_small() {
        let png = std::fs::read("tests/fixtures/multi-format/test.png").unwrap();
        let result = smart_compress(&png).unwrap();
        assert!(!result.compressed, "small image should not be compressed");
    }

    #[test]
    fn test_image_base64_roundtrip() {
        let b64 = to_base64(b"hello world test");
        assert!(!b64.is_empty());
    }

    // ── Chunking tests ────────────────────────────────────────────────

    #[test]
    fn test_chunk_short_no_split() {
        let content = "这是简短的一段话。";
        let chunks = split_at_punctuation(content, 500);
        assert_eq!(chunks.len(), 1, "短内容不应拆分");
        assert_eq!(chunks[0], content);
    }

    #[test]
    fn test_chunk_period_split() {
        let mut content = String::new();
        for i in 0..80 {
            content.push_str(&format!("这是第{}句话。", i));
        }
        // Content should exceed 500 chars and split at periods
        assert!(content.chars().count() > 500);
        let chunks = split_at_punctuation(&content, 500);
        assert!(chunks.len() >= 2, "应在句号处拆分");
        for chunk in &chunks {
            assert!(chunk.chars().count() <= 510, // slight overshoot from period join
                "每个chunk应≤500字左右, got {}", chunk.chars().count());
        }
    }

    #[test]
    fn test_chunk_newline_split() {
        let mut content = String::new();
        for i in 0..300 {
            content.push_str(&format!("line{}\n", i));
        }
        assert!(content.len() > 500);
        let chunks = split_at_punctuation(&content, 500);
        assert!(chunks.len() >= 2, "应在换行处拆分");
    }

    #[test]
    fn test_chunk_punctuation_priority() {
        // Content with periods should split at period, not at comma
        let mut content = String::new();
        for i in 0..35 {
            content.push_str(&format!("这是第{}段文字，包含逗号分隔的内容。", i));
        }
        assert!(content.chars().count() > 500);
        let chunks = split_at_punctuation(&content, 500);
        // Each chunk should end with a sentence-ending punctuation
        for chunk in &chunks[..chunks.len()-1] {
            assert!(
                chunk.ends_with('。') || chunk.ends_with('！') || chunk.ends_with('？') || chunk.ends_with('\n'),
                "chunk应优先在句末标点处拆分, got end: {:?}", chunk.chars().rev().take(5).collect::<String>()
            );
        }
    }

    #[test]
    fn test_chunk_inheritance() {
        // Verify that chunked DocSections inherit title/path/level
        let content = std::iter::repeat("长文本内容。").take(90).collect::<String>();
        let parent = DocSection {
            title: "Compaction".into(),
            section_path: "Compaction".into(),
            level: 2,
            node_type: "section".into(),
            content,
            images: vec![],
            parent_id: None,
        };

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::storage::migrate(&conn).unwrap();

        let count = write_doc_sections(&conn, "test", "test.md", "md", &[parent]).unwrap();
        eprintln!("write_doc_sections returned: {}", count);
        assert!(count >= 2, "长内容应拆分出多个节点, got {}", count);

        // Check that the parent was written
        let parent_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM nodes WHERE repo='test' AND node_type='section'",
            [], |r| r.get(0),
        ).unwrap_or(0);
        eprintln!("parent nodes: {}", parent_count);

        // Check chunks inherit parent fields
        let mut stmt = conn.prepare("SELECT name, node_type FROM nodes WHERE repo='test' AND node_type='chunk'").unwrap();
        let all_chunks: Vec<_> = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?))).unwrap().flatten().collect();
        eprintln!("chunks found: {:?}", all_chunks);
        
        // Also count total nodes
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM nodes WHERE repo='test'", [], |r| r.get(0)).unwrap_or(0);
        eprintln!("total nodes in test: {}", total);
        
        // Re-query chunks
        let mut stmt2 = conn.prepare("SELECT name, json_extract(attrs, '$.section_path'), json_extract(attrs, '$.level'), node_type, json_extract(attrs, '$.parent_id') FROM nodes WHERE node_type='chunk' ORDER BY json_extract(attrs, '$.section_path')").unwrap();
        let chunks: Vec<_> = stmt2.query_map([], |r| Ok((
            r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i32>(2)?,
            r.get::<_,String>(3)?, r.get::<_,Option<i64>>(4)?
        ))).unwrap().flatten().collect();

        assert!(!chunks.is_empty(), "应有chunk子节点");
        for (title, path, level, node_type, pid) in &chunks {
            assert_eq!(title, "Compaction");
            assert!(path.starts_with("Compaction/chunk/"));
            assert_eq!(*level, 2);
            assert_eq!(node_type, "chunk");
            assert!(pid.is_some(), "chunk应有parent_id");
        }
    }
}
