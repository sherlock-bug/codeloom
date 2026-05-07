// Unified document section IR — all parsers output this
#[derive(Debug, Clone)]
pub struct DocSection {
    pub title: String,
    pub section_path: String,
    pub level: i32,
    pub node_type: String,
    pub content: String,
    pub images: Vec<ImageRef>,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ImageRef {
    pub alt_text: String,
    pub original_src: String,       // original reference (URL or file path)
    pub raw_bytes: Vec<u8>,         // raw image bytes (empty for linked/remote)
    pub position: i32,              // order in document
    pub section_context: String,    // text context around image (100 chars before/after)
    pub image_type: String,         // "inline", "attachment", "linked"
}

impl ImageRef {
    pub fn new(alt_text: &str, original_src: &str, position: i32) -> Self {
        ImageRef {
            alt_text: alt_text.to_string(),
            original_src: original_src.to_string(),
            raw_bytes: Vec::new(),
            position,
            section_context: String::new(),
            image_type: "inline".to_string(),
        }
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.raw_bytes = bytes;
        self
    }

    pub fn linked(alt_text: &str, url: &str, position: i32) -> Self {
        ImageRef {
            alt_text: alt_text.to_string(),
            original_src: url.to_string(),
            raw_bytes: Vec::new(),
            position,
            section_context: String::new(),
            image_type: "linked".to_string(),
        }
    }
}
