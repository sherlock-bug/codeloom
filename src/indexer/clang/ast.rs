//! Clang AST JSON parsing and symbol/edge extraction.
//!
//! Extracts 15 node types and 11 edge types from `clang -ast-dump=json` output.

use sha2::{Sha256, Digest};
use crate::storage::symbols::{Symbol, make_sid};
use crate::{log_debug};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

// ─── Line number cache (offset→line conversion) ─────────────────────

static LINE_CACHE: OnceLock<Mutex<HashMap<String, Vec<u32>>>> = OnceLock::new();

fn offset_to_line(file_path: &str, offset: u32) -> u32 {
    let cache = LINE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().unwrap();
    let offsets = guard.entry(file_path.to_string()).or_insert_with(|| {
        let content = std::fs::read_to_string(file_path).unwrap_or_default();
        let mut off = Vec::with_capacity(content.len() / 40 + 1);
        off.push(0);
        for (i, b) in content.bytes().enumerate() {
            if b == b'\n' {
                off.push(i as u32 + 1);
            }
        }
        off
    });
    match offsets.binary_search(&offset) {
        Ok(line) => line as u32 + 1,
        Err(line) => line as u32,
    }
}

// ─── Extraction result ───────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct Edge {
    pub source_name: String,
    pub source_ns: String,
    pub target_name: String,
    pub edge_type: String,
}

pub struct ExtractionResult {
    pub symbols: Vec<Symbol>,
    pub edges: Vec<Edge>,
}

// ─── Main extraction entry ───────────────────────────────────────────

/// Extract symbols and edges from a Clang AST JSON tree.
pub fn extract_symbols_and_edges(
    ast: &serde_json::Value,
    repo_name: &str,
    file_path: &str,
    project_root: &str,
    ignore_patterns: &[String],
) -> ExtractionResult {
    let mut ctx = ExtractCtx {
        result: ExtractionResult { symbols: Vec::new(), edges: Vec::new() },
        repo: repo_name.to_string(),
        file: file_path.to_string(),
        project_root: project_root.to_string(),
        ignore_patterns: ignore_patterns.to_vec(),
        current_class: String::new(),
        current_access: String::new(),
        cur_file: file_path.to_string(),
        seen_strings: HashMap::new(),
        seen_symbols: Vec::new(),
    };

    let start = Instant::now();
    log_debug!("indexer::clang::ast", "extracting from {} (repo={})", file_path, repo_name);

    if let Some(inner) = ast.get("inner").and_then(|v| v.as_array()) {
        for (i, node) in inner.iter().enumerate() {
            let k = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            let n = node.get("name").and_then(|v| v.as_str());
            ctx.extract_node(node, "", None);
        }
    } else {
    }

    let elapsed = start.elapsed();
    log_debug!("indexer::clang::ast", "extraction done: {} symbols, {} edges in {}ms",
        ctx.result.symbols.len(), ctx.result.edges.len(), elapsed.as_millis());
    ctx.result
}

struct ExtractCtx {
    result: ExtractionResult,
    repo: String,
    file: String,
    project_root: String,
    ignore_patterns: Vec<String>,
    current_class: String,
    current_access: String,
    /// Current file context — tracks Clang's loc.file changes across includes
    cur_file: String,
    seen_strings: HashMap<String, bool>,
    seen_symbols: Vec<(String, String, String)>, // (name, ns, kind) for dedup
}

impl ExtractCtx {
    fn extract_node(&mut self, node: &serde_json::Value, parent_ns: &str, parent_class: Option<&str>) {
        let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        let name = node.get("name").and_then(|v| v.as_str());
        let loc = get_loc(node);
        let (ls, le) = get_range(node, &self.cur_file);

        // Get Clang's actual file path for this declaration
        // Clang only emits loc.file when the file context changes (e.g., after #include).
        // For nodes in the same file, loc.file is absent — we track cur_file to handle this.
        let loc_obj = node.get("loc");
        let decl_file = loc_obj
            .and_then(|v| v.get("file"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let has_loc = loc_obj.map_or(false, |v| !v.as_object().map_or(true, |o| o.is_empty()));
        if !decl_file.is_empty() {
            self.cur_file = decl_file.to_string();
        }
        // Compute namespace
        let ns = match kind {
            "NamespaceDecl" => {
                if let Some(n) = name {
                    if parent_ns.is_empty() { n.to_string() }
                    else { format!("{}::{}", parent_ns, n) }
                } else {
                    parent_ns.to_string()
                }
            }
            _ => parent_ns.to_string(),
        };

        // External if: compiler builtin (no loc), or non-project file
        // Header decls (loc without file) use the TU file for project check
        let is_external = if decl_file.is_empty() && !has_loc {
            true // compiler builtin with no source location
        } else if decl_file.is_empty() {
            !self.is_project_file(&self.cur_file) // header decl, use current file context
        } else {
            !self.is_project_file(decl_file)
        };
        if name.is_some() {
            log_debug!("indexer::clang::ast", "is_external={} name={:?} kind={} decl_file=\"{}\" cur_file=\"{}\" root=\"{}\"",
                is_external, name, kind, decl_file, self.cur_file, self.project_root);
        }

        if name.map_or(false, |n| n == "MyStruct" || n == "implicit_public" || n == "x") {
        }
        match kind {
            "FunctionDecl" | "CXXMethodDecl" => {
                if let Some(n) = name {
                    let implicit = node.get("isImplicit").and_then(|v| v.as_bool()).unwrap_or(false);
                    if implicit { return; }
                    
                    let sig = build_signature(node);
                    let is_def = has_body(node);
                    let access = self.current_access.clone();
                    let is_virtual = node.get("isVirtual").and_then(|v| v.as_bool()).unwrap_or(false);

                    let k = if kind == "CXXMethodDecl" { "method" } else { "function" };
                    let qname = if k == "method" {
                        if let Some(p) = parent_class {
                            format!("{}::{}", p, n)
                        } else {
                            n.to_string()
                        }
                    } else {
                        n.to_string()
                    };

                    let hash = hash_content(&qname, &sig, &decl_file, ls);

                    let mut sym = Symbol::default();
                    sym.repo = self.repo.clone();
                    sym.name = qname.clone();
                    sym.kind = k.to_string();
                    sym.content_hash = hash;
                    sym.file_path = if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() };
                    sym.line_start = ls;
                    sym.line_end = le;
                    sym.language = Some("cpp".to_string());
                    sym.signature = Some(sig.clone());
                    sym.namespace = if ns.is_empty() { None } else { Some(ns.clone()) };
                    sym.parent_class = if k == "method" { parent_class.map(|s| s.to_string()) } else { None };
                    sym.doc_comment = String::new();
                    sym.is_definition = is_def;
                    sym.access = access;
                    sym.is_virtual = is_virtual;
                    sym.is_external = is_external;
                    sym.sid = Some(make_sid(&self.repo, &qname, &sig, &ns, k, &sym.file_path));
                    self.result.symbols.push(sym);

                    // edges: only extract from internal (project) symbols
                    if !is_external {
                        self.extract_calls(node, &qname, &ns, &sig);
                        self.extract_param_types(node, &qname, &ns);
                        self.extract_return_type(node, &qname, &ns);
                        self.extract_variable_uses(node, &qname, &ns);
                    }

                    // overrides edge
                    if is_virtual || node.get("isOverride").is_some() {
                        if let Some(parent_method) = find_override_target(node) {
                            let pm = parent_method.clone();
                            self.result.edges.push(Edge {
                                source_name: qname.clone(), source_ns: ns.clone(),
                                target_name: parent_method,
                                edge_type: format!("overrides:{}", pm),
                            });
                        }
                    }
                }
            }

            "CXXRecordDecl" | "ClassTemplateDecl" => {
                if let Some(n) = name {
                    let tag = node.get("tagUsed").and_then(|v| v.as_str()).unwrap_or("class");
                    let k = if tag == "struct" { "struct" } else { "class" };
                    let default_access = if tag == "struct" { "public" } else { "private" };
                    self.current_access = default_access.to_string();
                    let cname = n.to_string();
                    let hash = hash_content(n, k, &self.file, ls);
                    
                    let mut sym = Symbol::default();
                    sym.repo = self.repo.clone();
                    sym.name = n.to_string();
                    sym.kind = k.to_string();
                    sym.content_hash = hash;
                    sym.file_path = if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() };
                    sym.line_start = ls;
                    sym.line_end = le;
                    sym.language = Some("cpp".to_string());
                    sym.namespace = if ns.is_empty() { None } else { Some(ns.clone()) };
                    sym.doc_comment = String::new();
                    sym.is_external = is_external;
                    sym.sid = Some(make_sid(
                        &self.repo, n, "", &ns, k, &sym.file_path));
                    self.add_symbol(sym, n, "", &ns, k);

                    // inherits edges
                    if let Some(bases) = node.get("bases").and_then(|v| v.as_array()) {
                        for base in bases {
                            if let Some(base_type) = base.get("type")
                                .and_then(|v| v.get("qualType"))
                                .and_then(|v| v.as_str())
                            {
                                let base_name = strip_cv_ref(base_type);
                                self.result.edges.push(Edge {
                                    source_name: n.to_string(), source_ns: ns.clone(),
                                    target_name: base_name.to_string(),
                                    edge_type: format!("inherits:{}", base_name),
                                });
                            }
                        }
                    }

                    // Process members to extract contains edges + method symbols
                    let prev_class = self.current_class.clone();
                    self.current_class = n.to_string();
                    if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
                        for child in inner {
                            let child_kind = child.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                            
                            // Track access specifier: AccessSpecDecl nodes carry the current access level
                            if child_kind == "AccessSpecDecl" {
                                if let Some(a) = child.get("access").and_then(|v| v.as_str()) {
                                    self.current_access = a.to_string();
                                }
                                continue;
                            }
                            
                            let child_name = child.get("name").and_then(|v| v.as_str());
                            
                            // contains: edge for methods and fields (qualified names)
                            if matches!(child_kind, "CXXMethodDecl" | "FieldDecl") {
                                if let Some(cn) = child_name {
                                    let qcn = format!("{}::{}", n, cn);
                                    self.result.edges.push(Edge {
                                        source_name: n.to_string(), source_ns: ns.clone(),
                                        target_name: qcn.clone(),
                                        edge_type: format!("contains:{}", qcn),
                                    });
                                }
                            }
                            
                            self.extract_node(child, &ns, Some(&cname));
                        }
                    }
                    self.current_class = prev_class;
                }
            }

            "FieldDecl" => {
                if let Some(n) = name {
                    let access = self.current_access.clone();
                    let field_type = extract_type_string(node);
                    let qname = if let Some(p) = parent_class {
                        format!("{}::{}", p, n)
                    } else {
                        n.to_string()
                    };
                    let hash = hash_content(&qname, "field", &self.file, ls);
                    
                    let sym = Symbol {
                        id: None, repo: self.repo.clone(), name: qname.clone(), kind: "field".to_string(),
                        content_hash: hash, file_path: if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() },
                        line_start: ls, line_end: le, language: Some("cpp".to_string()),
                        signature: None, parent_class: parent_class.map(|s| s.to_string()),
                        namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                        doc_comment: String::new(),
                        access: access.clone(),
                        is_external,
                        ..Default::default()
                    };
                    self.add_symbol(sym, &qname, "", &ns, "field");

                    // uses_type edge
                    if !field_type.is_empty() && !is_builtin_type(&field_type) {
                        let tname = strip_cv_ref(&field_type);
                        self.result.edges.push(Edge {
                            source_name: qname.clone(), source_ns: ns.clone(),
                            target_name: tname.to_string(),
                            edge_type: format!("uses_type:{}", tname),
                        });
                    }
                }
            }

            "VarDecl" => {
                if let Some(n) = name {
                    let storage = node.get("storageClass").and_then(|v| v.as_str()).unwrap_or("");
                    let is_global_or_static = storage == "static" || loc.0 == 0;
                    
                    let k = if storage == "static" { "static_var" }
                        else if is_global_or_static { "global" }
                        else { return; }; // Skip local variables

                    let var_type = extract_type_string(node);
                    let hash = hash_content(n, k, &decl_file, ls);

                    let sym = Symbol {
                        id: None, repo: self.repo.clone(), name: n.to_string(), kind: k.to_string(),
                        content_hash: hash, file_path: if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() },
                        line_start: ls, line_end: le, language: Some("cpp".to_string()),
                        signature: None, parent_class: None,
                        namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                        doc_comment: String::new(),
                        is_external,
                    ..Default::default()
                    };
                    self.add_symbol(sym, n, "", &ns, k);

                    // uses_type edge
                    if !var_type.is_empty() && !is_builtin_type(&var_type) {
                        let tname = strip_cv_ref(&var_type);
                        self.result.edges.push(Edge {
                            source_name: n.to_string(), source_ns: ns.clone(),
                            target_name: tname.to_string(),
                            edge_type: format!("uses_type:{}", tname),
                        });
                    }
                }
            }

            "EnumDecl" => {
                if let Some(n) = name {
                    let hash = hash_content(n, "enum", &self.file, ls);
                    let sym = Symbol {
                        id: None, repo: self.repo.clone(), name: n.to_string(), kind: "enum".to_string(),
                        content_hash: hash, file_path: if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() },
                        line_start: ls, line_end: le, language: Some("cpp".to_string()),
                        signature: None, parent_class: None,
                        namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                        doc_comment: String::new(),
                        is_external,
                    ..Default::default()
                    };
                    self.add_symbol(sym, n, "", &ns, "enum");
                }
                // enum values are inside as EnumConstantDecl
            }

            "EnumConstantDecl" => {
                if let Some(n) = name {
                    let qname = if let Some(p) = parent_class {
                        format!("{}::{}", p, n)
                    } else {
                        n.to_string()
                    };
                    let hash = hash_content(&qname, "enum_value", &self.file, ls);
                    let sym = Symbol {
                        id: None, repo: self.repo.clone(), name: qname.clone(), kind: "enum_value".to_string(),
                        content_hash: hash, file_path: if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() },
                        line_start: ls, line_end: le, language: Some("cpp".to_string()),
                        signature: None, parent_class: parent_class.map(|s| s.to_string()),
                        namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                        doc_comment: String::new(),
                        is_external,
                    ..Default::default()
                    };
                    self.add_symbol(sym, &qname, "", &ns, "enum_value");
                }
            }

            "NamespaceDecl" => {
                if let Some(n) = name {
                    if n != "(anonymous)" && !n.is_empty() {
                        let hash = format!("ns:{}", n);
                        let sym = Symbol {
                            id: None, repo: self.repo.clone(), name: n.to_string(), kind: "namespace".to_string(),
                            content_hash: hash, file_path: if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() },
                            line_start: ls, line_end: le, language: Some("cpp".to_string()),
                            signature: None, parent_class: None,
                            namespace: None, doc_comment: String::new(),
                            is_external,
                    ..Default::default()
                        };
                        self.add_symbol(sym, n, "", "", "namespace");
                    }
                }
            }

            "FunctionTemplateDecl" => {
                // Function template: process children directly.
                // First FunctionDecl child is the primary template declaration.
                // Subsequent FunctionDecl children are template instantiations.
                if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
                    let mut first_func = true;
                    for child in inner {
                        let child_kind = child.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                        if child_kind == "FunctionDecl" || child_kind == "CXXMethodDecl" {
                            if let Some(cn) = child.get("name").and_then(|v| v.as_str()) {
                                let sig = build_signature(child);
                                let is_def = has_body(child);
                                let tkind = if first_func { "template_function" } else { "template_instance" };
                                first_func = false;

                                let hash = hash_content(cn, tkind, &self.file, ls);
                                let tsym = Symbol {
                                    id: None, repo: self.repo.clone(), name: cn.to_string(),
                                    kind: tkind.to_string(), content_hash: hash,
                                    file_path: String::new(),  // template: no single definition file
                                    line_start: get_range(child, &self.cur_file).0, line_end: get_range(child, &self.cur_file).1,
                                    language: Some("cpp".to_string()),
                                    signature: Some(sig.clone()),
                                    namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                                    doc_comment: String::new(),
                                    is_definition: is_def, is_external: false,
                                ..Default::default()
                                };
                                self.add_symbol(tsym, cn, &sig, &ns, tkind);

                                // instantiates edge for template instances
                                if tkind == "template_instance" {
                                    self.result.edges.push(Edge {
                                        source_name: cn.to_string(), source_ns: ns.clone(),
                                        target_name: cn.to_string(),
                                        edge_type: format!("instantiates:{}", cn),
                                    });
                                }
                            }
                        } else {
                            // Process non-function children normally (TemplateTypeParmDecl, etc.)
                            self.extract_node(child, &ns, parent_class);
                        }
                    }
                }
                return;  // children already processed, skip general recursion
            }

            "ClassTemplateSpecializationDecl" => {
                if let Some(n) = name {
                    // Check if primary template is in project files
                    let primary = node.get("specializedTemplate")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    // Simple heuristic: if name starts with "std::" it's external
                    if primary.starts_with("std::") {
                        // STL template instance — skip
                    } else {
                        let hash = hash_content(n, "template_instance", &self.file, ls);
                        let sym = Symbol {
                            id: None, repo: self.repo.clone(), name: n.to_string(),
                            kind: "template_instance".to_string(),
                            content_hash: hash, file_path: String::new(),  // template instance: no single definition file
                            line_start: ls, line_end: le, language: Some("cpp".to_string()),
                            signature: None, parent_class: None,
                            namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                            doc_comment: String::new(),
                            is_external: false,  // template instances are project symbols
                    ..Default::default()
                        };
                        self.add_symbol(sym, n, "", &ns, "template_instance");

                        // instantiates edge
                        let tname = strip_template_args(n);
                        if !tname.is_empty() {
                            self.result.edges.push(Edge {
                                source_name: n.to_string(), source_ns: ns.clone(),
                                target_name: tname.to_string(),
                                edge_type: format!("instantiates:{}", tname),
                            });
                        }
                    }
                }
            }

            "TypedefDecl" | "TypeAliasDecl" => {
                if let Some(n) = name {
                    let hash = hash_content(n, "typedef", &self.file, ls);
                    let sym = Symbol {
                        id: None, repo: self.repo.clone(), name: n.to_string(), kind: "typedef".to_string(),
                        content_hash: hash, file_path: if decl_file.is_empty() { self.cur_file.clone() } else { decl_file.to_string() },
                        line_start: ls, line_end: le, language: Some("cpp".to_string()),
                        signature: None, parent_class: None,
                        namespace: if ns.is_empty() { None } else { Some(ns.clone()) },
                        doc_comment: String::new(),
                        is_external,
                    ..Default::default()
                    };
                    self.add_symbol(sym, n, "", &ns, "typedef");

                    // aliases edge
                    let underlying = extract_type_string(node);
                    if !underlying.is_empty() {
                        let tname = strip_cv_ref(&underlying);
                        self.result.edges.push(Edge {
                            source_name: n.to_string(), source_ns: ns.clone(),
                            target_name: tname.to_string(),
                            edge_type: format!("aliases:{}", tname),
                        });
                    }
                }
            }

            "StringLiteral" => {
                let value = node.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !value.is_empty() && value.len() <= 128 {
                    let key = value.to_string();
                    if !self.seen_strings.contains_key(&key) {
                        self.seen_strings.insert(key.clone(), true);
                        let hash = format!("str:{:x}", Sha256::digest(value.as_bytes())).chars().take(16).collect();
                        let mut sym = Symbol::default();
                        sym.repo = self.repo.clone();
                        sym.name = value.to_string();
                        sym.kind = "string_literal".to_string();
                        sym.content_hash = hash;
                        sym.file_path = String::new();
                        sym.line_start = 0;
                        sym.line_end = 0;
                        sym.language = Some("cpp".to_string());
                        sym.sid = Some(make_sid(&self.repo, value, "", "", "string_literal", ""));
                        self.result.symbols.push(sym);
                    }
                }
            }

            "PreprocessedEntity" | " InclusionDirective" => {
                // #include edge — extract file-level dependency
                if let Some(included) = node.get("file").and_then(|v| v.get("file")).and_then(|v| v.as_str()) {
                    let fname = std::path::Path::new(included)
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if !fname.is_empty() {
                        let fname2 = fname.clone();
                        let src_file = std::path::Path::new(&self.file)
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        self.result.edges.push(Edge {
                            source_name: src_file, source_ns: String::new(),
                            target_name: fname,
                            edge_type: format!("includes:{}", fname2),
                        });
                    }
                }
            }

            _ => {}
        }

        // Skip recursion into function bodies and expressions (belt-and-suspenders:
        // the Python filter already strips these, but direct clang calls won't)
        if is_body_kind(kind) {
            return;
        }

        // Recurse into children unconditionally
        if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
            let child_ns = if kind == "NamespaceDecl" { &ns } else { parent_ns };
            for child in inner {
                let child_parent = if matches!(kind, "CXXRecordDecl" | "ClassTemplateDecl" | "EnumDecl") {
                    name
                } else {
                    parent_class
                };
                self.extract_node(child, child_ns, child_parent);
            }
        }
    }

    fn is_project_file(&self, file: &str) -> bool {
        if file.is_empty() {
            return false;
        }
        // Resolve relative paths before matching: Clang emits file paths
        // relative to the TU (e.g. "db/version_set.h") which won't start_with.
        let abs_file = if std::path::Path::new(file).is_relative() {
            std::path::Path::new(&self.project_root).join(file).to_string_lossy().to_string()
        } else {
            file.to_string()
        };
        abs_file.starts_with(&self.project_root)
            && !crate::ignore::is_ignored(&abs_file, &self.ignore_patterns)
    }

    // ─── Edge extraction helpers ───────────────────────────────────

    fn extract_calls(&mut self, node: &serde_json::Value, caller: &str, ns: &str, _sig: &str) {
        extract_call_targets(node, |callee| {
            let c = callee.clone();
            self.result.edges.push(Edge {
                source_name: caller.to_string(), source_ns: ns.to_string(),
                target_name: callee,
                edge_type: format!("calls:{}", c),
            });
        });
    }

    fn extract_param_types(&mut self, node: &serde_json::Value, func: &str, ns: &str) {
        if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
            for child in inner {
                if child.get("kind").and_then(|v| v.as_str()) == Some("ParmVarDecl") {
                    let ptype = extract_type_string(child);
                    if !ptype.is_empty() && !is_builtin_type(&ptype) {
                        let tname = strip_cv_ref(&ptype);
                        self.result.edges.push(Edge {
                            source_name: func.to_string(), source_ns: ns.to_string(),
                            target_name: tname.to_string(),
                            edge_type: format!("param_type:{}", tname),
                        });
                    }
                }
            }
        }
    }

    fn extract_return_type(&mut self, node: &serde_json::Value, func: &str, ns: &str) {
        if let Some(type_info) = node.get("type").and_then(|v| v.get("qualType")).and_then(|v| v.as_str()) {
            if let Some(ret) = type_info.split('(').next() {
                let ret = ret.trim();
                if !ret.is_empty() && ret != "void" && !is_builtin_type(ret) {
                    let tname = strip_cv_ref(ret);
                    self.result.edges.push(Edge {
                        source_name: func.to_string(), source_ns: ns.to_string(),
                        target_name: tname.to_string(),
                        edge_type: format!("return_type:{}", tname),
                    });
                }
            }
        }
    }

    fn extract_variable_uses(&mut self, node: &serde_json::Value, func: &str, ns: &str) {
        extract_decl_refs(node, |ref_name| {
            let rn = ref_name.clone();
            self.result.edges.push(Edge {
                source_name: func.to_string(), source_ns: ns.to_string(),
                target_name: ref_name,
                edge_type: format!("uses:{}", rn),
            });
        });
    }

    // ─── Helpers ────────────────────────────────────────────────────

    fn add_symbol(&mut self, sym: Symbol, name: &str, _sig: &str, ns: &str, kind: &str) {
        // Dedup by (name, namespace, kind) for external symbols
        let key = (name.to_string(), ns.to_string(), kind.to_string());
        if self.seen_symbols.iter().any(|(n, s, k)| n == &key.0 && s == &key.1 && k == &key.2) {
            return;
        }
        self.seen_symbols.push(key);
        self.result.symbols.push(sym);
    }
}

// ─── Free helper functions ───────────────────────────────────────────

fn get_loc(node: &serde_json::Value) -> (u32, u32) {
    let loc = node.get("loc");
    let line = loc.and_then(|v| v.get("line")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let col = loc.and_then(|v| v.get("col")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    (line, col)
}

fn get_range(node: &serde_json::Value, file_path: &str) -> (u32, u32) {
    let range = node.get("range");
    let begin = range.and_then(|v| v.get("begin"));
    let end = range.and_then(|v| v.get("end"));

    // Try range.begin.line first (rarely present in Clang 18), fallback to offset→line
    let ls = begin.and_then(|v| v.get("line")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let ls = if ls > 0 {
        ls
    } else if let Some(off) = begin.and_then(|v| v.get("offset")).and_then(|v| v.as_u64()) {
        offset_to_line(file_path, off as u32)
    } else {
        0
    };

    // line_end: try range.end.line, then offset→line, then fallback to ls
    let le = end.and_then(|v| v.get("line")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let le = if le > 0 {
        le
    } else if let Some(off) = end.and_then(|v| v.get("offset")).and_then(|v| v.as_u64()) {
        offset_to_line(file_path, off as u32)
    } else {
        ls
    };

    (ls, le)
}

fn build_signature(node: &serde_json::Value) -> String {
    node.get("type")
        .and_then(|v| v.get("qualType"))
        .and_then(|v| v.as_str())
        .unwrap_or("?")
        .to_string()
}

fn has_body(node: &serde_json::Value) -> bool {
    fn check(n: &serde_json::Value) -> bool {
        if n.get("kind").and_then(|v| v.as_str()) == Some("CompoundStmt") {
            return true;
        }
        if let Some(inner) = n.get("inner").and_then(|v| v.as_array()) {
            inner.iter().any(check)
        } else { false }
    }
    check(node)
}

fn extract_type_string(node: &serde_json::Value) -> String {
    node.get("type")
        .and_then(|v| v.get("qualType"))
        .and_then(|v| v.as_str())
        .map(|t| strip_cv_ref(t).to_string())
        .unwrap_or_default()
}

fn strip_cv_ref(t: &str) -> &str {
    let t = t.trim();
    let t = t.strip_prefix("const ").unwrap_or(t);
    let t = t.strip_suffix(" &").unwrap_or(t);
    let t = t.strip_suffix(" *").unwrap_or(t);
    let t = t.strip_suffix(" **").unwrap_or(t);
    let t = t.strip_prefix("class ").unwrap_or(t);
    let t = t.strip_prefix("struct ").unwrap_or(t);
    let t = t.strip_prefix("enum ").unwrap_or(t);
    // Remove angle bracket content for template types (simplified)
    if let Some(pos) = t.find('<') {
        &t[..pos]
    } else { t }
}

fn is_builtin_type(t: &str) -> bool {
    matches!(t, "void" | "int" | "unsigned int" | "long" | "unsigned long"
        | "short" | "unsigned short" | "char" | "unsigned char"
        | "float" | "double" | "bool" | "_Bool" | "size_t" | "ssize_t"
        | "int8_t" | "uint8_t" | "int16_t" | "uint16_t" | "int32_t" | "uint32_t"
        | "int64_t" | "uint64_t")
}

fn strip_template_args(name: &str) -> &str {
    if let Some(pos) = name.find('<') {
        &name[..pos]
    } else {
        name
    }
}

fn find_override_target(node: &serde_json::Value) -> Option<String> {
    // Look for the base method in referenced declaration
    node.get("referencedDecl")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn extract_call_targets<F: FnMut(String)>(node: &serde_json::Value, mut cb: F) {
    fn walk(n: &serde_json::Value, cb: &mut dyn FnMut(String)) {
        let kind = n.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        if kind == "DeclRefExpr" {
            if let Some(ref_decl) = n.get("referencedDecl") {
                if let Some(ref_name) = ref_decl.get("name").and_then(|v| v.as_str()) {
                    cb(ref_name.to_string());
                }
            }
        }
        if let Some(inner) = n.get("inner").and_then(|v| v.as_array()) {
            for child in inner {
                walk(child, cb);
            }
        }
    }
    walk(node, &mut cb);
}

fn extract_decl_refs<F: FnMut(String)>(node: &serde_json::Value, mut cb: F) {
    extract_call_targets(node, |name| cb(name));
}

fn hash_content(name: &str, sig: &str, file: &str, line: u32) -> String {
    let mut h = Sha256::new();
    h.update(name.as_bytes());
    h.update(sig.as_bytes());
    h.update(file.as_bytes());
    h.update(&line.to_le_bytes());
    format!("{:x}", h.finalize()).chars().take(16).collect()
}

/// Check if a Clang AST node kind is a function body/expression (not a declaration).
/// Used as belt-and-suspenders: the Python filter already strips these,
/// but direct clang calls won't.
fn is_body_kind(kind: &str) -> bool {
    matches!(kind,
        "CompoundStmt" | "IfStmt" | "ForStmt" | "WhileStmt" | "DoStmt" | "SwitchStmt"
        | "ReturnStmt" | "DeclStmt" | "BreakStmt" | "ContinueStmt" | "GotoStmt"
        | "CallExpr" | "CXXMemberCallExpr" | "CXXOperatorCallExpr"
        | "BinaryOperator" | "UnaryOperator" | "ConditionalOperator"
        | "ImplicitCastExpr" | "CXXStaticCastExpr" | "CXXDynamicCastExpr"
        | "CXXReinterpretCastExpr" | "CXXConstCastExpr" | "CStyleCastExpr"
        | "DeclRefExpr" | "MemberExpr" | "CXXDependentScopeMemberExpr"
        | "UnresolvedLookupExpr" | "CXXThisExpr" | "CXXNullPtrLiteralExpr"
        | "IntegerLiteral" | "FloatingLiteral" | "CharacterLiteral"
        | "ArraySubscriptExpr" | "InitListExpr" | "CXXConstructExpr"
        | "MaterializeTemporaryExpr" | "CXXBindTemporaryExpr"
        | "CXXFunctionalCastExpr" | "ParenExpr" | "CXXNewExpr" | "CXXDeleteExpr"
        | "NullStmt" | "LabelStmt" | "CaseStmt" | "DefaultStmt"
        | "WarnUnusedResultAttr" | "AlwaysInlineAttr" | "VisibilityAttr"
        | "FullComment" | "ParagraphComment" | "TextComment"
        | "AccessSpecDecl"
    )
}

