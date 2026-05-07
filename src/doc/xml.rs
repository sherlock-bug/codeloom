// XML/HTML parser: extract text content by element hierarchy
use crate::doc::section::{DocSection, ImageRef};
use quick_xml::Reader;
use quick_xml::events::Event;

pub fn parse_xml(content: &str) -> anyhow::Result<Vec<DocSection>> {
    let mut reader = Reader::from_str(content);
    let mut buf = Vec::new();
    let mut sections: Vec<DocSection> = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut current_text = String::new();
    let mut in_body = false;
    let mut is_html = false;
    let mut images = Vec::new();
    let mut image_pos = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if name == "html" { is_html = true; }
                if is_html && name == "body" { in_body = true; }
                if is_html && (name == "script" || name == "style" || name == "noscript") {
                    // Skip script/style/noscript content
                    reader.read_to_end_into(e.name(), &mut Vec::new())?;
                    continue;
                }
                // img tag in HTML
                if name == "img" {
                    let mut alt = String::new();
                    let mut src = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_lowercase();
                        let val = String::from_utf8_lossy(&attr.value);
                        if key == "alt" { alt = val.to_string(); }
                        if key == "src" { src = val.to_string(); }
                    }
                    if !src.is_empty() {
                        image_pos += 1;
                        let mut img = ImageRef::new(&alt, &src, image_pos);
                        img.image_type = if src.starts_with("http") { "linked".to_string() } else { "inline".to_string() };
                        img.section_context = current_text.chars().rev().take(100).collect::<String>().chars().rev().collect();
                        images.push(img);
                    }
                }
                path.push(name);
                // For non-HTML XML, start collecting text at top-level elements
                if !is_html && path.len() == 1 {
                    if !current_text.trim().is_empty() && !sections.is_empty() {
                        sections.last_mut().unwrap().content = std::mem::take(&mut current_text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                if is_html && name == "body" { in_body = false; }
                // On closing a top-level element (XML) or section-level (HTML), create section
                let create_section = (!is_html && path.len() == 2 && name == path[1])
                    || (is_html && in_body && (name == "div" || name == "section" || name == "article" || name == "p"));
                
                if create_section && !current_text.trim().is_empty() {
                    let section_path = if is_html { String::new() } else { format!("/{}", path[1..].join("/")) };
                    sections.push(DocSection {
                        title: path.last().cloned().unwrap_or_default(),
                        section_path,
                        level: 2,
                        node_type: "section".to_string(),
                        content: std::mem::take(&mut current_text),
                        images: std::mem::take(&mut images),
                    });
                    image_pos = 0;
                }
                path.pop();
            }
            Ok(Event::Text(ref e)) => {
                let text = e.decode()?;
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    if !current_text.is_empty() { current_text.push(' '); }
                    current_text.push_str(trimmed);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                // Gracefully handle XML errors
                if e.to_string().contains("UnexpectedEof") { break; }
            }
            _ => {}
        }
        buf.clear();
    }

    // Final section if content remains
    if !current_text.trim().is_empty() {
        sections.push(DocSection {
            title: "root".to_string(),
            section_path: if is_html { String::new() } else { "/".to_string() },
            level: 1,
            node_type: "section".to_string(),
            content: std::mem::take(&mut current_text),
            images: std::mem::take(&mut images),
        });
    }

    // If no sections created, make one with all text
    if sections.is_empty() && is_html {
        sections.push(DocSection {
            title: "html".to_string(),
            section_path: String::new(),
            level: 1,
            node_type: "section".to_string(),
            content: "HTML document".to_string(),
            images: vec![],
        });
    }

    Ok(sections)
}
