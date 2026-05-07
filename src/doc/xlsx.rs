// Excel parser: smart header detection + four-layer node model (sheet/header_cell/row/cell)
use crate::doc::section::{DocSection, ImageRef};
use calamine::{open_workbook_auto_from_rs, Reader, Data};

pub fn parse_xlsx(bytes: &[u8]) -> anyhow::Result<Vec<DocSection>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| anyhow::anyhow!("Failed to open Excel: {}", e))?;

    let mut sections = Vec::new();
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();

    for sheet_name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(sheet_name) {
            let rows: Vec<Vec<String>> = range.rows()
                .map(|r| r.iter().map(|c| cell_to_string(c)).collect())
                .collect();

            if rows.is_empty() || rows.iter().all(|r| r.iter().all(|c| c.trim().is_empty())) {
                continue; // skip empty sheets
            }

            // Smart header detection: find first row with text-heavy content
            let header_idx = detect_header(&rows);
            let headers = rows[header_idx].clone();

            // Sheet meta node (level=1)
            let safe_sheet = sanitize_path(sheet_name);
            let total_rows = if header_idx + 1 < rows.len() { rows.len() - header_idx - 1 } else { 0 };
            sections.push(DocSection {
                title: sheet_name.clone(),
                section_path: safe_sheet.clone(),
                level: 1,
                node_type: "sheet".to_string(),
                content: format!("列名: {}  共{}行", headers.iter().filter(|c| !c.trim().is_empty()).cloned().collect::<Vec<_>>().join(", "), total_rows),
                images: vec![],
            });

            // Header cells (level=2)
            for col_name in &headers {
                let cn = col_name.trim();
                if cn.is_empty() { continue; }
                let safe_col = sanitize_path(cn);
                sections.push(DocSection {
                    title: cn.to_string(),
                    section_path: format!("{}/_header/{}", safe_sheet, safe_col),
                    level: 2,
                    node_type: "header_cell".to_string(),
                    content: cn.to_string(),
                    images: vec![],
                });
            }

            // Data rows and cells
            for (i, row) in rows.iter().enumerate().skip(header_idx + 1) {
                if row.iter().all(|c| c.trim().is_empty()) { continue; }
                let row_num = i - header_idx; // 1-indexed data row

                // Row node (level=3)
                let row_path = format!("{}/Row{}", safe_sheet, row_num);
                sections.push(DocSection {
                    title: format!("Row{}", row_num),
                    section_path: row_path.clone(),
                    level: 3,
                    node_type: "row".to_string(),
                    content: String::new(),
                    images: vec![],
                });

                // Cell nodes (level=4) — each cell is searchable
                for (j, val) in row.iter().enumerate() {
                    let v = val.trim();
                    if v.is_empty() { continue; }
                    let col_name = if j < headers.len() { headers[j].trim() } else { &format!("列{}", char::from_u32(65 + j as u32).unwrap_or('?')) };
                    let safe_col = sanitize_path(col_name);
                    if safe_col.is_empty() { continue; }
                    sections.push(DocSection {
                        title: col_name.to_string(),
                        section_path: format!("{}/{}/{}", safe_sheet, row_path, safe_col),
                        level: 4,
                        node_type: "cell".to_string(),
                        content: v.to_string(),
                        images: vec![],
                    });
                }
            }

            // Embedded images from xl/media/ (if any)
            // calamine doesn't directly expose images, but we can check
            if let Some(media) = extract_xlsx_images(bytes, sheet_name) {
                // Images attached to sheet meta node
                if let Some(sheet_node) = sections.iter_mut().rev().find(|s| s.node_type == "sheet" && s.title == *sheet_name) {
                    sheet_node.images = media;
                }
            }
        }
    }

    Ok(sections)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 { format!("{}", *f as i64) } else { format!("{}", f) }
        }
        Data::Int(i) => format!("{}", i),
        Data::Bool(b) => format!("{}", b),
        Data::DateTime(d) => format!("{}", d),
        Data::DateTimeIso(d) => format!("{}", d),
        Data::DurationIso(d) => format!("{}", d),
        Data::Error(e) => format!("ERROR:{}", e),
        Data::Empty => String::new(),
    }
}

/// Detect header row: find row with highest text-to-numeric ratio
fn detect_header(rows: &[Vec<String>]) -> usize {
    let scan = rows.len().min(50);
    let mut best_idx = 0;
    let mut best_score = 0.0;

    for i in 0..scan {
        let non_empty: Vec<&String> = rows[i].iter().filter(|c| !c.trim().is_empty()).collect();
        if non_empty.is_empty() { continue; }
        let text_count = non_empty.iter().filter(|c| c.trim().chars().any(|ch| ch.is_alphabetic())).count();
        let score = text_count as f64 / non_empty.len() as f64;
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }

    // If no text-heavy row found, use first non-empty row
    if best_score == 0.0 {
        best_idx = rows.iter().position(|r| r.iter().any(|c| !c.trim().is_empty())).unwrap_or(0);
    }

    best_idx
}

/// Extract embedded images from xlsx (xl/media/ directory in zip)
fn extract_xlsx_images(bytes: &[u8], _sheet_name: &str) -> Option<Vec<ImageRef>> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut images = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        let name = file.name().to_string();
        if name.starts_with("xl/media/") && (name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".gif")) {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buf).ok()?;
            let pos = images.len() as i32 + 1;
            let fname = name.rsplit('/').next().unwrap_or(&name);
            let mut img = ImageRef::new("", fname, pos);
            img.raw_bytes = buf;
            img.image_type = "attachment".to_string();
            images.push(img);
        }
    }

    if images.is_empty() { None } else { Some(images) }
}

fn sanitize_path(s: &str) -> String {
    s.replace('/', "_").replace('\\', "_").replace('|', "_")
}
