// PDF parser: extract text content
use crate::doc::section::DocSection;
use pdf_extract::extract_text;

pub fn parse_pdf(bytes: &[u8]) -> anyhow::Result<Vec<DocSection>> {
    // 10MB limit
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(anyhow::anyhow!("PDF too large (max 10MB)"));
    }

    let text = extract_text_from_memory(bytes)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(vec![]); // empty/scanned PDF, skip
    }

    // Split by pages (pdf-extract joins pages with form feeds or double newlines)
    let joined = trimmed.split('\n').collect::<Vec<_>>()
        .join("\n");
    let pages: Vec<&str> = joined
        .split("\n\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut sections = Vec::new();
    if pages.is_empty() {
        // Single section
        sections.push(DocSection {
            title: "PDF".to_string(),
            section_path: String::new(),
            level: 1,
            node_type: "section".to_string(),
            content: trimmed.to_string(),
            images: vec![],
        });
    } else {
        for (i, page) in pages.iter().enumerate() {
            if page.trim().is_empty() { continue; }
            sections.push(DocSection {
                title: format!("第{}页", i + 1),
                section_path: format!("第{}页", i + 1),
                level: 2,
                node_type: "section".to_string(),
                content: page.to_string(),
                images: vec![],
            });
        }
    }

    Ok(sections)
}

fn extract_text_from_memory(bytes: &[u8]) -> anyhow::Result<String> {
    // pdf-extract reads from file path, not memory. Write to temp file.
    let tmp = std::env::temp_dir().join(format!("codeloom_pdf_{}.pdf", std::process::id()));
    std::fs::write(&tmp, bytes)?;
    let result = extract_text(&tmp);
    let _ = std::fs::remove_file(&tmp);
    result.map_err(|e| anyhow::anyhow!("PDF extraction failed: {}", e))
}
