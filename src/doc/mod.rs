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
        "xml" | "html" | "htm" => {
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
    let mut count = 0;
    let mut last_node_id;

    for sec in sections {
        let hash = crate::storage::dedup::hash_content(
            &format!("{}:{}:{}", sec.title, sec.section_path, sec.content)
        );
        let sp = if sec.section_path.is_empty() { String::new() } else { sec.section_path.clone() };

        conn.execute(
            "INSERT INTO doc_nodes (repo, title, section_path, content, level, file_path, file_format, content_hash, node_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(repo, file_path, section_path) DO UPDATE SET
             title=excluded.title, content=excluded.content, level=excluded.level,
             file_format=excluded.file_format, content_hash=excluded.content_hash, node_type=excluded.node_type",
            rusqlite::params![repo, sec.title, sp, sec.content, sec.level, path, file_format, hash, sec.node_type],
        )?;
        count += 1;
        last_node_id = conn.last_insert_rowid();

        if !sec.images.is_empty() {
            crate::storage::insert_doc_images(conn, last_node_id, &sec.images)?;
        }
    }

    Ok(count)
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
}
