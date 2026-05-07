// DOCX parser: extract paragraphs, heading styles, embedded images
use crate::doc::section::{DocSection, ImageRef};
use quick_xml::Reader;
use quick_xml::events::Event;

pub fn parse_docx(bytes: &[u8]) -> anyhow::Result<Vec<DocSection>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // Read document.xml
    let doc_xml = {
        let mut file = archive.by_name("word/document.xml")
            .map_err(|_| anyhow::anyhow!("word/document.xml not found in docx"))?;
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut file, &mut buf)?;
        buf
    };

    let mut reader = Reader::from_str(&doc_xml);
    let mut buf = Vec::new();
    let mut sections = Vec::new();
    let mut current_text = String::new();
    let mut current_heading = String::new();
    let mut current_level = 0;
    let mut in_paragraph = false;
    let mut section_idx = 0;
    let mut images = Vec::new();
    let mut image_pos = 0;
    let mut para_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name().as_ref().to_owned();
                let name = String::from_utf8_lossy(&name_bytes);
                if name == "w:p" {
                    in_paragraph = true;
                    para_text.clear();
                } else if name == "w:pStyle" {
                    // Detect heading style
                    if let Some(val) = e.attributes().flatten().find(|a| String::from_utf8_lossy(a.key.as_ref()) == "w:val") {
                        let style = String::from_utf8_lossy(&val.value);
                        if style.starts_with("Heading") || style.starts_with("heading") || style.starts_with("Title") {
                            current_heading = String::new();
                            current_level = match &*style {
                                s if s.contains("1") => 1,
                                s if s.contains("2") => 2,
                                s if s.contains("3") => 3,
                                _ => 1,
                            };
                        }
                    }
                } else if name == "w:drawing" || name == "w:pict" {
                    // Embedded image — record position, extract later
                    if !para_text.is_empty() {
                        image_pos += 1;
                        let mut img = ImageRef::new("", &format!("embedded_{}.png", image_pos), image_pos);
                        img.section_context = current_text.chars().rev().take(100).collect::<String>().chars().rev().collect();
                        img.image_type = "attachment".to_string();
                        images.push(img);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().as_ref().to_owned();
                let name = String::from_utf8_lossy(&name_bytes);
                if name == "w:p" {
                    in_paragraph = false;
                    let pt = para_text.trim().to_string();
                    para_text.clear();

                    if !pt.is_empty() {
                        if !current_heading.is_empty() {
                            // Heading paragraph → flush previous section
                            flush_section(&mut sections, &mut current_text, &mut current_heading, current_level, &mut section_idx, &mut images);
                            current_heading = pt.clone();
                        } else if current_text.len() + pt.len() > 2000 {
                            // Auto-split on large content
                            flush_section(&mut sections, &mut current_text, &mut current_heading, current_level, &mut section_idx, &mut images);
                            current_heading = String::new();
                            current_text = pt;
                        } else {
                            if !current_text.is_empty() { current_text.push('\n'); }
                            current_text.push_str(&pt);
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.decode()?;
                if in_paragraph { para_text.push_str(&text); }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Flush final section
    flush_section(&mut sections, &mut current_text, &mut current_heading, current_level, &mut section_idx, &mut images);

    // Extract actual image bytes from word/media/
    extract_docx_images(&mut archive, &mut sections);

    Ok(sections)
}

fn flush_section(
    sections: &mut Vec<DocSection>,
    text: &mut String,
    heading: &mut String,
    level: i32,
    idx: &mut i32,
    images: &mut Vec<ImageRef>,
) {
    let content = text.trim().to_string();
    if content.is_empty() && heading.is_empty() { return; }

    *idx += 1;
    let section_path = if heading.is_empty() {
        format!("第{}部分", idx)
    } else {
        let h = heading.trim();
        if *idx > 1 { format!("{}-{}", h, idx) } else { h.to_string() }
    };

    sections.push(DocSection {
        title: if heading.is_empty() { format!("第{}部分", idx) } else { heading.clone() },
        section_path,
        level: if level == 0 { 2 } else { level },
        node_type: "section".to_string(),
        content,
        images: std::mem::take(images),
    });

    text.clear();
    *heading = String::new();
}

fn extract_docx_images(archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>, sections: &mut [DocSection]) {
    // Collect rId→filename mapping from rels
    let mut rid_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Ok(mut rels_file) = archive.by_name("word/_rels/document.xml.rels") {
        let mut rels_xml = String::new();
        if std::io::Read::read_to_string(&mut rels_file, &mut rels_xml).is_ok() {
            let mut reader = Reader::from_str(&rels_xml);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                        if String::from_utf8_lossy(e.name().as_ref()) == "Relationship" {
                            let mut id = String::new();
                            let mut target = String::new();
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref());
                                let val = String::from_utf8_lossy(&attr.value);
                                if key == "Id" { id = val.to_string(); }
                                if key == "Target" { target = val.to_string(); }
                            }
                            if target.starts_with("media/") {
                                rid_map.insert(id, target);
                            }
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
        }
    }

    // Read image bytes from word/media/
    for section in sections.iter_mut() {
        for img in &mut section.images {
            if !img.raw_bytes.is_empty() { continue; }
            let lookup = if img.original_src.starts_with("media/") {
                img.original_src.clone()
            } else {
                rid_map.get(&img.original_src).cloned().unwrap_or_default()
            };
            if lookup.is_empty() { continue; }
            let media_path = format!("word/{}", lookup);
            if let Ok(mut media_file) = archive.by_name(&media_path) {
                let mut buf = Vec::new();
                if std::io::Read::read_to_end(&mut media_file, &mut buf).is_ok() {
                    img.raw_bytes = buf;
                }
            }
        }
    }
}
