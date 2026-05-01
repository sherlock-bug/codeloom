// C++ extraction using tree-sitter Node API with field names from node-types.json
use tree_sitter::Node;
use crate::indexer::tree_sitter::{extract_text, FileInfo};
use crate::storage::dedup;
use crate::storage::symbols::Symbol;

pub fn extract(
    source: &str, root: Node, file: &FileInfo, repo: &str,
    symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
) { walk_children(source, &root, file, repo, None, symbols, edges); }

fn walk_children(
    source: &str, node: &Node, file: &FileInfo, repo: &str,
    parent_class: Option<&str>, symbols: &mut Vec<Symbol>,
    edges: &mut Vec<(usize, usize, String)>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "function_definition" => extract_func(source, &child, file, repo, parent_class, symbols, edges),
            "class_specifier" | "struct_specifier" => extract_class(source, &child, file, repo, symbols, edges),
            "enum_specifier" => extract_enum(source, &child, file, repo, symbols),
            "field_declaration" => extract_field(source, &child, file, repo, parent_class, symbols, edges),
            "declaration" => {
                let mut dc = child.walk();
                let children: Vec<_> = child.children(&mut dc).collect();
                let has_body = children.iter().any(|c| c.kind() == "compound_statement");
                let has_func = children.iter().any(|c|
                    matches!(c.kind(), "function_definition"|"class_specifier"|"struct_specifier"|"enum_specifier")
                );
                if has_body {
                    extract_func(source, &child, file, repo, parent_class, symbols, edges);
                } else if has_func {
                    walk_children(source, &child, file, repo, parent_class, symbols, edges);
                } else {
                    extract_decl(source, &child, file, repo, parent_class, symbols);
                }
            }
            "template_declaration" | "namespace_definition" | "linkage_specification" => {
                let body = child.child_by_field_name("body")
                    .or_else(|| {
                        let children: Vec<_> = {
                            let mut c = child.walk();
                            child.children(&mut c).collect()
                        };
                        children.into_iter().find(|n|
                            matches!(n.kind(), "function_definition"|"class_specifier"|"struct_specifier"|"enum_specifier"|"declaration"|"field_declaration")
                        )
                    });
                if let Some(body) = body {
                    match body.kind() {
                        "function_definition" => extract_func(source, &body, file, repo, parent_class, symbols, edges),
                        "class_specifier" | "struct_specifier" => extract_class(source, &body, file, repo, symbols, edges),
                        "enum_specifier" => extract_enum(source, &body, file, repo, symbols),
                        "field_declaration" => extract_field(source, &body, file, repo, parent_class, symbols, edges),
                        "declaration" => {
                            let mut dc = body.walk();
                            if body.children(&mut dc).any(|c| c.kind() == "compound_statement") {
                                extract_func(source, &body, file, repo, parent_class, symbols, edges);
                            } else {
                                walk_children(source, &body, file, repo, parent_class, symbols, edges);
                            }
                        }
                        _ => walk_children(source, &body, file, repo, parent_class, symbols, edges),
                    }
                }
            }
            _ => {}
        }
    }
}

fn extract_func(
    source: &str, node: &Node, file: &FileInfo, repo: &str,
    parent_class: Option<&str>, symbols: &mut Vec<Symbol>,
    edges: &mut Vec<(usize, usize, String)>,
) {
    let declarator = node.child_by_field_name("declarator");
    let mut name = declarator
        .and_then(|d| d.child_by_field_name("declarator"))
        .or_else(|| declarator.and_then(|d| d.child(0)))
        .and_then(|n| Some(n.utf8_text(source.as_bytes()).ok()?.to_string()))
        .unwrap_or_default();
    let name_looks_bad = name.is_empty() || name.starts_with('(')
        || name.contains("HEDLEY") || name.contains("DEPRECATED");
    if name_looks_bad {
        if let Some(pc) = parent_class {
            name = pc.to_string();
        } else if let Some(ty) = node.child_by_field_name("type") {
            if let Ok(t) = ty.utf8_text(source.as_bytes()) {
                let t = t.to_string();
                if !t.contains("HEDLEY") && !t.contains("DEPRECATED") {
                    name = t;
                }
            }
        }
    }
    if name.is_empty() || name.contains("HEDLEY") { name = "anon".into(); }
    let full = match parent_class { Some(c) => format!("{}::{}", c, name), None => name.clone() };
    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let comment = collect_comments(source, node);
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: full.clone(),
        kind: if parent_class.is_some() {"method"} else {"function"}.into(),
        definition: def.clone(), content_hash: dedup::hash_content(&def),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: Some(full), parent_class: parent_class.map(|s| s.into()), namespace: None,
    });
    let idx = symbols.len() - 1;
    // contains edge: class → method
    if let Some(pc) = parent_class {
        emit_contains(symbols, pc, idx, &name, edges);
    }
    extract_calls(source, node, idx, edges);
    extract_type_edges(source, node, idx, edges);
}
/// Extract return type, parameter types, and override/virtual markers for a function
fn extract_type_edges(source: &str, node: &Node, func_idx: usize, edges: &mut Vec<(usize, usize, String)>) {
    // returns: return type
    if let Some(ty) = node.child_by_field_name("type") {
        if let Ok(t) = ty.utf8_text(source.as_bytes()) {
            let t = t.trim().to_string();
            if t != "void" && !t.is_empty() {
                edges.push((func_idx, usize::MAX, format!("returns:{}", t)));
            }
        }
    }
    // param_type: each parameter type
    let declarator = match node.child_by_field_name("declarator") {
        Some(d) => d,
        None => return,
    };
    if let Some(params) = declarator.child_by_field_name("parameters") {
        let mut pc = params.walk();
        for child in params.children(&mut pc) {
            if child.kind() == "parameter_declaration" {
                if let Some(ty) = child.child_by_field_name("type") {
                    if let Ok(t) = ty.utf8_text(source.as_bytes()) {
                        let t = t.trim().to_string();
                        if !t.is_empty() {
                            edges.push((func_idx, usize::MAX, format!("param_type:{}", t)));
                        }
                    }
                }
            }
        }
    }
    // overrides: detect 'override' via source text (tree-sitter-cpp unnamed node)
    // The function definition text contains " override" or ") override" for override methods
    if let Ok(fn_text) = node.utf8_text(source.as_bytes()) {
        if fn_text.contains("override") && !fn_text.contains("override default") {
            if let Some(name_node) = declarator.child_by_field_name("declarator")
                .or_else(|| declarator.child(0))
            {
                if let Ok(method_name) = name_node.utf8_text(source.as_bytes()) {
                    edges.push((func_idx, usize::MAX, format!("overrides:{}", method_name)));
                }
            }
        }
    }
}

fn extract_class(
    source: &str, node: &Node, file: &FileInfo, repo: &str,
    symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
) {
    let kind = if node.kind() == "struct_specifier" { "struct" } else { "class" };
    let name = node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("anonymous");
    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let comment = collect_comments(source, node);
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: name.into(), kind: kind.into(),
        definition: def.clone(), content_hash: dedup::hash_content(&def),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: Some(name.into()), parent_class: None, namespace: None,
    });
    let class_idx = symbols.len() - 1;

    // inheritance edges
    {
        let mut cursor2 = node.walk();
        for child in node.children(&mut cursor2) {
            if child.kind() == "base_class_clause" {
                let mut bc = child.walk();
                for bc_child in child.children(&mut bc) {
                    if matches!(bc_child.kind(), "qualified_identifier" | "template_type" | "type_identifier") {
                        if let Ok(n) = bc_child.utf8_text(source.as_bytes()) {
                            edges.push((class_idx, usize::MAX, format!("inherits:{}", n)));
                        }
                    }
                }
            }
        }
    }

    if let Some(body) = node.child_by_field_name("body") {
        walk_children(source, &body, file, repo, Some(name), symbols, edges);
    }
}

fn extract_enum(source: &str, node: &Node, file: &FileInfo, repo: &str, symbols: &mut Vec<Symbol>) {
    let name = node.child_by_field_name("name").and_then(|n| n.utf8_text(source.as_bytes()).ok()).unwrap_or("anonymous");
    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let comment = collect_comments(source, node);
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: name.into(), kind: "enum".into(),
        definition: def.clone(), content_hash: dedup::hash_content(&def),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: None, parent_class: None, namespace: None,
    });
}

/// Extract class member field (field_declaration in tree-sitter-cpp)
/// Captures: field name, parent class, field type → generates contains + field_type edges
fn extract_field(
    source: &str, node: &Node, file: &FileInfo, repo: &str,
    parent_class: Option<&str>, symbols: &mut Vec<Symbol>,
    edges: &mut Vec<(usize, usize, String)>,
) {
    // node-types.json: field_declaration has fields "type" and "declarator"
    let name = node.child_by_field_name("declarator")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("?");
    let type_name = node.child_by_field_name("type")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("?");

    let full_name = match parent_class {
        Some(pc) => format!("{}::{}", pc, name),
        None => name.to_string(),
    };

    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let comment = collect_comments(source, node);
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: full_name.clone(),
        kind: "field".into(),
        definition: def.clone(), content_hash: dedup::hash_content(&def),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: Some(format!("{}: {}", name, type_name)),
        parent_class: parent_class.map(|s| s.into()), namespace: None,
    });
    let field_idx = symbols.len() - 1;

    // contains edge: class → field
    if let Some(pc) = parent_class {
        emit_contains(symbols, pc, field_idx, name, edges);
    }
    // field_type edge: field → its type
    if type_name != "?" {
        edges.push((field_idx, usize::MAX, format!("field_type:{}", type_name)));
    }
}

fn extract_decl(source: &str, node: &Node, file: &FileInfo, repo: &str, parent_class: Option<&str>, symbols: &mut Vec<Symbol>) {
    if let Some(decl) = node.child_by_field_name("declarator") {
        if let Ok(name) = decl.utf8_text(source.as_bytes()) {
            let full = match parent_class {
                Some(pc) => format!("{}::{}", pc, name),
                None => name.to_string(),
            };
            let def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
            symbols.push(Symbol {
                id: None, repo: repo.into(), name: full,
                kind: "variable".into(),
                definition: def.clone(), content_hash: dedup::hash_content(&def),
                file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
                line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
                signature: None, parent_class: parent_class.map(|s| s.into()), namespace: None,
            });
        }
    }
}

/// Emit a contains edge from a class (found by name in symbols) to a member
fn emit_contains(symbols: &[Symbol], class_name: &str, member_idx: usize, member_name: &str, edges: &mut Vec<(usize, usize, String)>) {
    if let Some(ci) = symbols.iter().position(|s| s.name == class_name) {
        edges.push((ci, member_idx, format!("contains:{}", member_name)));
    }
}

/// Collect doc comments (///, /** */) preceding a node using source text scanning.
fn collect_comments(source: &str, node: &Node) -> String {
    let start_byte = node.start_byte();
    if start_byte == 0 { return String::new(); }
    let before = &source[..start_byte];
    let mut lines: Vec<&str> = before.lines().collect();
    let mut comments = Vec::new();
    // walk backwards from the line before the node
    while let Some(line) = lines.pop() {
        let trimmed = line.trim();
        if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**") || trimmed.starts_with(" *") || trimmed == "*/" {
            comments.push(trimmed);
        } else if trimmed.is_empty() {
            // blank line — stop unless we're in a block comment
            continue;
        } else {
            break; // non-comment, non-blank line — stop
        }
    }
    comments.reverse();
    if comments.is_empty() { return String::new(); }
    comments.join("\n")
}

fn extract_calls(source: &str, node: &Node, caller_idx: usize, edges: &mut Vec<(usize, usize, String)>) {
    let mut cursor = node.walk();
    loop {
        let cur = cursor.node();
        if cur.kind() == "call_expression" {
            if let Some(func) = cur.child_by_field_name("function") {
                if let Ok(name) = func.utf8_text(source.as_bytes()) {
                    let name = name.trim();
                    if name.len() > 1 && !matches!(name, "if"|"for"|"while"|"return"|"switch"|"sizeof"|"static_cast"|"reinterpret_cast"|"dynamic_cast"|"const_cast") {
                        edges.push((caller_idx, usize::MAX, format!("calls:{}", name)));
                    }
                }
            }
        }
        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() { return; }
            }
        }
    }
}
