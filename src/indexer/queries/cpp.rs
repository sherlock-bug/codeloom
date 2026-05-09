// C++ extraction using tree-sitter Node API with field names from node-types.json
use tree_sitter::Node;
use crate::indexer::tree_sitter::{extract_text, FileInfo};
use crate::storage::dedup;
use crate::storage::symbols::Symbol;

pub fn extract(
    source: &str, root: Node, file: &FileInfo, repo: &str,
    symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
) {
    // Full-tree string literal collection must run FIRST so that
    // extract_calls_with_types can find string_literal symbols.
    collect_all_string_literals(source, &root, file, repo, symbols);
    walk_children(source, &root, file, repo, None, symbols, edges);
}

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
            "enum_specifier" => extract_enum(source, &child, file, repo, symbols, edges),
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
                // Recurse into children for nested string_literals, macros, etc.
                walk_children(source, &child, file, repo, parent_class, symbols, edges);
            }
            "template_declaration" => {
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
                let sym_idx: Option<usize> = if let Some(body) = body {
                    // Collect comments from the template_declaration node (not body)
                    // so that comments before 'template<...>' are captured
                    let template_comment = collect_comments(source, &child);
                    let idx = match body.kind() {
                        "function_definition" => Some(extract_func_template(source, &body, file, repo, parent_class, symbols, edges, &template_comment)),
                        "class_specifier" | "struct_specifier" => Some(extract_class_template(source, &body, file, repo, symbols, edges, &template_comment)),
                        "enum_specifier" => { extract_enum(source, &body, file, repo, symbols, edges); None }
                        "field_declaration" => { extract_field(source, &body, file, repo, parent_class, symbols, edges); None }
                        "declaration" => {
                            let mut dc = body.walk();
                            if body.children(&mut dc).any(|c| c.kind() == "compound_statement") {
                                Some(extract_func_template(source, &body, file, repo, parent_class, symbols, edges, &template_comment))
                            } else {
                                walk_children(source, &body, file, repo, parent_class, symbols, edges);
                                None
                            }
                        }
                        _ => { walk_children(source, &body, file, repo, parent_class, symbols, edges); None }
                    };
                    idx
                } else {
                    None
                };
                if let Some(idx) = sym_idx {
                    extract_template_params(source, &child, idx, edges);
                }
            }
            "namespace_definition" | "linkage_specification" => {
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
                        "enum_specifier" => extract_enum(source, &body, file, repo, symbols, edges),
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
            "preproc_def" | "preproc_function_def" => extract_macro(source, &child, file, repo, symbols),
            "preproc_if" | "preproc_ifdef" | "preproc_else" => walk_children(source, &child, file, repo, parent_class, symbols, edges),
            "string_literal" | "raw_string_literal" | "concatenated_string" | "system_lib_string" => extract_string_literal(source, &child, file, repo, symbols),
            _ => {}
        }
    }
}

fn extract_func(
    source: &str, node: &Node, file: &FileInfo, repo: &str,
    parent_class: Option<&str>, symbols: &mut Vec<Symbol>,
    edges: &mut Vec<(usize, usize, String)>,
) {
    let kind = if parent_class.is_some() { "method" } else { "function" };
    extract_func_impl(source, node, file, repo, parent_class, symbols, edges, kind, "");
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
    extract_class_impl(source, node, file, repo, symbols, edges, kind, "");
}

fn extract_enum(source: &str, node: &Node, file: &FileInfo, repo: &str, symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>) {
    let name = node.child_by_field_name("name").and_then(|n| n.utf8_text(source.as_bytes()).ok()).unwrap_or("anonymous");
    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let mut comment = collect_comments(source, node);
    let body_comments = collect_body_comments(source, node);
    if !body_comments.is_empty() {
        if !comment.is_empty() { comment.push_str(" | "); }
        comment.push_str(&body_comments);
    }
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: name.into(), kind: "enum".into(),
        content_hash: dedup::hash_content(&name),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: None, parent_class: None, namespace: None,
    doc_comment: comment,
    });
    let enum_idx = symbols.len() - 1;
    // Extract enum values from enumerator_list
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "enumerator_list" {
            let mut ec = child.walk();
            for enumerator in child.children(&mut ec) {
                if enumerator.kind() == "enumerator" {
                    if let Some(vn) = enumerator.child_by_field_name("name") {
                        if let Ok(vname) = vn.utf8_text(source.as_bytes()) {
                            let full = format!("{}::{}", name, vname);
                            symbols.push(Symbol {
                                id: None, repo: repo.into(), name: full.clone(), kind: "enum_value".into(),
                                content_hash: String::new(),
                                file_path: file.path.clone(),
                                line_start: enumerator.start_position().row as u32+1,
                                line_end: enumerator.end_position().row as u32+1,
                                language: Some("cpp".into()), signature: Some(full),
                                parent_class: Some(name.to_string()), namespace: None,
                            doc_comment: String::new(),
                            });
                            let vi = symbols.len() - 1;
                            edges.push((enum_idx, vi, format!("contains:{}", vname)));
                        }
                    }
                }
            }
        }
    }
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
        content_hash: dedup::hash_content(&def),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: Some(format!("{}: {}", name, type_name)),
        parent_class: parent_class.map(|s| s.into()), namespace: None,
    doc_comment: comment,
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
    // Container/template edges (std::vector<B> → aggregate:B, MyVector<int> → template_use)
    if type_name != "?" {
        extract_container_edges(type_name, field_idx, symbols, edges);
    }
}

fn extract_decl(source: &str, node: &Node, file: &FileInfo, repo: &str, parent_class: Option<&str>, symbols: &mut Vec<Symbol>) {
    // Detect static storage class
    let is_static = {
        let mut dc = node.walk();
        let x = node.children(&mut dc).any(|c| c.kind() == "storage_class_specifier" && c.utf8_text(source.as_bytes()).map(|t| t.trim()=="static").unwrap_or(false));
        x
    };
    let is_global = parent_class.is_none();
    if let Some(decl) = node.child_by_field_name("declarator") {
        // Extract just the identifier (declarator utf8_text may include initializer like "x = 0")
        let decl_name = find_innermost_identifier(source, &decl)
            .unwrap_or_else(|| decl.utf8_text(source.as_bytes()).unwrap_or("?").to_string());
        let full = match parent_class {
            Some(pc) => format!("{}::{}", pc, decl_name),
            None => decl_name.to_string(),
        };
        let kind = if is_static { "static_var" } else if is_global { "global" } else { "variable" };
        let comment = collect_comments(source, node);
        symbols.push(Symbol {
            id: None, repo: repo.into(), name: full,
            kind: kind.into(),
            content_hash: dedup::hash_content(&decl_name),
            file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
            line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
            signature: None,parent_class: parent_class.map(|s| s.into()), namespace: None,
            doc_comment: comment,
        });
    }
}

/// Emit a contains edge from a class (found by name in symbols) to a member
fn emit_contains(symbols: &[Symbol], class_name: &str, member_idx: usize, member_name: &str, edges: &mut Vec<(usize, usize, String)>) {
    if let Some(ci) = symbols.iter().position(|s| s.name == class_name) {
        edges.push((ci, member_idx, format!("contains:{}", member_name)));
    }
}

/// Collect doc comments (///, /** */) preceding a node + inline // on same line.
fn collect_comments(source: &str, node: &Node) -> String {
    let start_byte = node.start_byte();
    let mut parts = Vec::new();

    // 1. Preceding doc comments
    if start_byte > 0 {
        let before = &source[..start_byte];
        let mut lines: Vec<&str> = before.lines().collect();
        let mut comments = Vec::new();
        while let Some(line) = lines.pop() {
            let trimmed = line.trim();
            if trimmed.starts_with("///") || trimmed.starts_with("//!") || trimmed.starts_with("/**") || trimmed.starts_with(" *") || trimmed == "*/" {
                comments.push(trimmed);
            } else if trimmed.is_empty() {
                continue;
            } else {
                break;
            }
        }
        comments.reverse();
        if !comments.is_empty() {
            parts.push(comments.join("\n"));
        }
    }

    // 2. Inline comment on same line (// ... after node text)
    let end_byte = node.end_byte();
    let rest_of_line = &source[end_byte..];
    if let Some(comment_start) = rest_of_line.find("//") {
        let before_comment = &rest_of_line[..comment_start];
        if before_comment.trim().is_empty() {
            let inline = rest_of_line[comment_start..]
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .trim_start_matches("//")
                .trim();
            if !inline.is_empty() {
                parts.push(inline.to_string());
            }
        }
    }

    parts.join(" | ")
}

/// Collect comment nodes inside a function/class body (tree-sitter 'comment' nodes).
fn collect_body_comments(source: &str, node: &Node) -> String {
    let mut comments = Vec::new();
    collect_comments_recursive(source, node, &mut comments);
    if comments.is_empty() {
        return String::new();
    }
    comments.join(" | ")
}

fn collect_comments_recursive(source: &str, node: &Node, comments: &mut Vec<String>) {
    if node.kind() == "comment" {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                comments.push(trimmed.to_string());
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_comments_recursive(source, &child, comments);
    }
}

/// Scan function body for local variable declarations and parameters.
/// Returns HashMap<variable_name, type_name>.
fn scan_local_declarations(source: &str, func_node: &Node, symbols: &[Symbol]) -> std::collections::HashMap<String, String> {
    let mut types: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // 1. Function parameters: walk the declarator's parameter list
    if let Some(decl) = func_node.child_by_field_name("declarator") {
        if let Some(params) = decl.child_by_field_name("parameters") {
            let mut pc = params.walk();
            for child in params.children(&mut pc) {
                if child.kind() == "parameter_declaration" {
                    let ty_text = child.child_by_field_name("type")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.trim().to_string());
                    let var_name = param_var_name(source, &child);
                    if let (Some(ty), Some(var)) = (ty_text, var_name) {
                        if !ty.is_empty() && !var.is_empty() {
                            types.insert(var, simplify_type(&ty));
                        }
                    }
                }
            }
        }
    }

    // 2. Local declarations in function body
    if let Some(body) = func_node.child_by_field_name("body") {
        scan_declarations_in_node(source, &body, &mut types, symbols);
    } else {
        // Some function_definition nodes don't have a body field — scan the whole node
        scan_declarations_in_node(source, func_node, &mut types, symbols);
    }

    types
}

fn param_var_name(source: &str, param: &tree_sitter::Node) -> Option<String> {
    // Try declarator field first
    if let Some(decl) = param.child_by_field_name("declarator") {
        // Walk to find the deepest identifier (skip pointer/reference/array wrappers)
        let name = find_innermost_identifier(source, &decl);
        if name.is_some() { return name; }
    }
    // Fallback: walk children for identifier/pointer_declarator/reference_declarator
    let mut cursor = param.walk();
    for child in param.children(&mut cursor) {
        if matches!(child.kind(), "identifier" | "field_identifier") {
            return child.utf8_text(source.as_bytes()).ok().map(|s| s.trim().to_string());
        }
        if matches!(child.kind(), "pointer_declarator" | "reference_declarator") {
            let name = find_innermost_identifier(source, &child);
            if name.is_some() { return name; }
        }
    }
    None
}

fn find_innermost_identifier(source: &str, node: &tree_sitter::Node) -> Option<String> {
    if matches!(node.kind(), "identifier" | "field_identifier") {
        return node.utf8_text(source.as_bytes()).ok().map(|s| s.trim().to_string());
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(name) = find_innermost_identifier(source, &child) {
            return Some(name);
        }
    }
    None
}

/// Walk a node recursively collecting variable declarations
fn scan_declarations_in_node(source: &str, node: &tree_sitter::Node,
    types: &mut std::collections::HashMap<String, String>, symbols: &[Symbol],
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "declaration" {
            let ty_text = child.child_by_field_name("type")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .map(|s| s.trim().to_string());
            let decl_node = child.child_by_field_name("declarator");
            if let (Some(ty), Some(decl)) = (ty_text, decl_node) {
                let var_name = find_innermost_identifier(source, &decl);
                if let Some(var) = var_name {
                    if !var.is_empty() {
                        // Handle "auto x = new Foo()" — extract Foo
                        let resolved = if ty == "auto" {
                            try_resolve_auto(source, &child, symbols)
                        } else {
                            simplify_type(&ty)
                        };
                        types.insert(var, resolved);
                    }
                }
            }
        }
        // Recurse into compound_statement / body nodes (skip nested function/class to avoid cross-scope)
        if !matches!(child.kind(), "function_definition" | "class_specifier" | "struct_specifier" | "lambda_expression") {
            scan_declarations_in_node(source, &child, types, symbols);
        }
    }
}

/// Simplify type: strip pointer, reference, const, volatile
fn simplify_type(ty: &str) -> String {
    let mut s = ty.to_string();
    // Remove leading const/volatile/struct/enum/class
    for prefix in &["const ", "volatile ", "struct ", "enum ", "class "] {
        s = s.strip_prefix(prefix).unwrap_or(&s).to_string();
    }
    // Remove trailing *, &, const, volatile
    s.trim_end_matches(|c: char| c == '*' || c == '&' || c == ' ').to_string()
    // Remove trailing const/volatile after pointer
    .trim_end_matches(" const").trim_end_matches(" volatile").to_string()
}

/// Try to resolve "auto x = new Foo()" -> "Foo"
fn try_resolve_auto(_source: &str, _decl_node: &tree_sitter::Node, symbols: &[Symbol]) -> String {
    // 1. Simple cases: walk children for "new" expression
    let mut cursor = _decl_node.walk();
    for child in _decl_node.children(&mut cursor) {
        if child.kind() == "new_expression" {
            if let Some(ty) = child.child_by_field_name("type") {
                if let Ok(t) = ty.utf8_text(_source.as_bytes()) {
                    let t = t.trim().to_string();
                    if !t.is_empty() { return simplify_type(&t); }
                }
            }
        }
        // 2. call_expression: auto x = func() — extract func name, look up signature
        if child.kind() == "call_expression" {
            if let Some(func_node) = child.child_by_field_name("function") {
                if let Ok(func_name) = func_node.utf8_text(_source.as_bytes()) {
                    let func_name = func_name.trim();
                    // Try to find the function in symbols and parse its return type
                    if let Some(ret) = resolve_return_type(func_name, symbols) {
                        if !ret.is_empty() { return ret; }
                    }
                }
            }
        }
    }
    "auto".to_string()
}

/// Look up a function in symbols and extract its return type from signature.
/// Signature format: "TypeName* func(args)" — return type is everything before the last space.
fn resolve_return_type(func_name: &str, symbols: &[Symbol]) -> Option<String> {
    for sym in symbols {
        if sym.name == func_name || sym.name.ends_with(&format!("::{}", func_name)) {
            if let Some(sig) = &sym.signature {
                // Signature: "ReturnType func_name(params)" → extract ReturnType
                let sig = sig.trim();
                // Find the function name in the signature to split before it
                if let Some(pos) = sig.find(func_name) {
                    let ret = sig[..pos].trim();
                    if !ret.is_empty() {
                        return Some(simplify_type(ret));
                    }
                }
            }
        }
    }
    None
}

/// Extract call edges, enum value usage, global variable references,
/// and string literal associations from a function body in one AST walk.
fn extract_calls_with_types(
    source: &str,
    func_node: &Node,
    caller_idx: usize,
    edges: &mut Vec<(usize, usize, String)>,
    parent_class: Option<&str>,
    symbols: &[Symbol],
) {
    // Build type table from local declarations + params.
    let types = scan_local_declarations(source, func_node, symbols);
    let local_names: std::collections::HashSet<String> = types.keys().cloned().collect();

    let mut cursor = func_node.walk();
    loop {
        let cur = cursor.node();
        match cur.kind() {
            // ── Call extraction (original logic) ──────────────────────
            "call_expression" => {
                if let Some(func) = cur.child_by_field_name("function") {
                    let name = if func.kind() == "field_expression" {
                        let obj = func.child_by_field_name("argument");
                        let field = func.child_by_field_name("field");
                        let obj_name = obj
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.trim().to_string());
                        let method_name = field
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.trim().to_string());
                        match (obj_name, method_name) {
                            (Some(obj), Some(method)) if obj == "this" => match parent_class {
                                Some(pc) => format!("{}::{}", pc, method),
                                None => method,
                            },
                            (Some(obj), Some(method)) => match types.get(&obj) {
                                Some(ty) => format!("{}::{}", ty, method),
                                None => method,
                            },
                            _ => func
                                .utf8_text(source.as_bytes())
                                .unwrap_or("?")
                                .trim()
                                .to_string(),
                        }
                    } else {
                        let bare = func
                            .utf8_text(source.as_bytes())
                            .unwrap_or("?")
                            .trim()
                            .to_string();
                        if let Some(pc) = parent_class {
                            let qualified = format!("{}::{}", pc, bare);
                            if symbols.iter().any(|s| s.name == qualified) {
                                qualified
                            } else {
                                bare
                            }
                        } else {
                            bare
                        }
                    };
                    let is_noise = matches!(
                        name.as_str(),
                        "if" | "for"
                            | "while"
                            | "return"
                            | "switch"
                            | "sizeof"
                            | "static_cast"
                            | "reinterpret_cast"
                            | "dynamic_cast"
                            | "const_cast"
                    );
                    if name.len() > 1 && !is_noise {
                        let arg_count = if let Some(args) = cur.child_by_field_name("arguments") {
                            args.children(&mut args.walk())
                                .filter(|c| c.kind() != "," && c.kind() != "(" && c.kind() != ")")
                                .count()
                        } else {
                            0
                        };
                        let edge_label = format!("calls:{}({})", name, arg_count);
                        edges.push((caller_idx, usize::MAX, edge_label));
                        // Virtual dispatch
                        let method_suffix = if let Some(colon) = name.rfind("::") {
                            &name[colon + 2..]
                        } else {
                            name.as_str()
                        };
                        let override_edge = format!("overrides:{}", method_suffix);
                        if edges.iter().any(|(_, _, label)| label == &override_edge) {
                            for sym in symbols {
                                if sym.name.ends_with(&format!("::{}", method_suffix))
                                    && sym.name != name
                                    && sym
                                        .signature
                                        .as_ref()
                                        .map_or(false, |s| s.contains("override"))
                                {
                                    let ov_label =
                                        format!("calls_override:{}({})", sym.name, arg_count);
                                    edges.push((caller_idx, usize::MAX, ov_label));
                                }
                            }
                        }
                    }
                }
            }

            // ── Enum value usage: qualified_identifier → EnumName::Value ─
            "qualified_identifier" => {
                if let Ok(text) = cur.utf8_text(source.as_bytes()) {
                    let text = text.trim();
                    let colons = text.matches("::").count();
                    if colons >= 1 && colons <= 2 {
                        // Check: is the full text an enum_value symbol?
                        let enum_idx = symbols
                            .iter()
                            .position(|s| s.kind == "enum_value" && s.name == text);
                        if let Some(idx) = enum_idx {
                            edges.push((caller_idx, idx, format!("uses:{}", text)));
                        // For colons==1, also try matching just the suffix as enum_value
                        } else if colons == 1 {
                            if let Some(pos) = text.rfind("::") {
                                let suffix = &text[pos + 2..];
                                let enum_idx = symbols.iter().position(|s| {
                                    s.kind == "enum_value"
                                        && (s.name == suffix
                                            || s.name.ends_with(&format!("::{}", suffix)))
                                });
                                if let Some(idx) = enum_idx {
                                    edges.push((caller_idx, idx, format!("uses:{}", text)));
                                }
                            }
                        }
                    }
                }
            }

            // ── Global / static variable reference: bare identifier ────
            "identifier" => {
                if let Ok(name) = cur.utf8_text(source.as_bytes()) {
                    let name = name.trim();
                    if !local_names.contains(name)
                        && name.len() > 1
                        && !matches!(
                            name,
                            "if" | "for"
                                | "while"
                                | "return"
                                | "sizeof"
                                | "true"
                                | "false"
                                | "nullptr"
                                | "this"
                                | "switch"
                                | "case"
                                | "default"
                                | "break"
                                | "continue"
                                | "void"
                                | "int"
                                | "bool"
                                | "char"
                                | "float"
                                | "double"
                                | "auto"
                                | "const"
                                | "static"
                                | "new"
                                | "delete"
                                | "class"
                                | "struct"
                                | "enum"
                                | "namespace"
                                | "public"
                                | "private"
                                | "protected"
                                | "virtual"
                                | "override"
                        )
                    {
                        if let Some(idx) = symbols.iter().position(|s| {
                            (s.kind == "global" || s.kind == "static_var") && s.name == name
                        }) {
                            edges.push((caller_idx, idx, format!("references:{}", name)));
                        }
                    }
                }
            }

            _ => {}
        }

        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    // ── Walk complete: string literal association via text search ─
                    if let Ok(body_text) = func_node.utf8_text(source.as_bytes()) {
                        for sym in symbols {
                            if sym.kind == "string_literal" && body_text.contains(sym.name.as_str()) {
                                edges.push((caller_idx, sym.id.map(|id| id as usize).unwrap_or(usize::MAX),
                                    format!("uses:{}", sym.name)));
                            }
                        }
                    }
                    return;
                }
            }
        }
    }
}

// ── Template extraction ──────────────────────────────────────────────

fn resolve_or_max(name: &str, symbols: &[Symbol]) -> usize {
    if name.is_empty() || name == "?" { return usize::MAX; }
    symbols.iter().position(|s| {
        s.name == name || s.name.ends_with(&format!("::{}", name))
    }).unwrap_or(usize::MAX)
}

/// Map std container names to their relationship edge types.
/// Returns None for non-std types or types we don't handle.
fn std_container_edge(qualified_type: &str, inner_type: &str) -> Option<(String, Option<String>)> {
    // Normalize: extract the base container name, stripping std:: prefix and template args
    let base = qualified_type.trim().strip_prefix("std::").unwrap_or(qualified_type);
    // Get just the container name (before '<')
    let cname = base.split('<').next().unwrap_or(base).trim();
    match cname {
        "vector" | "list" | "deque" | "set" | "unordered_set" | "multiset"
        | "stack" | "queue" | "priority_queue" | "array" | "forward_list"
        | "unordered_multiset" | "span" | "initializer_list"
        => Some((format!("aggregate:{}", inner_type), None)),
        "unique_ptr" => Some((format!("owns:{}", inner_type), None)),
        "shared_ptr" | "weak_ptr" => Some((format!("shares:{}", inner_type), None)),
        "map" | "unordered_map" | "multimap" | "unordered_multimap" => {
            // For maps, inner_type is "K,V" — split into key and value
            let parts: Vec<&str> = inner_type.split(',').map(|s| s.trim()).collect();
            let key = parts.first().copied().unwrap_or(inner_type);
            let val = parts.get(1).copied().unwrap_or(inner_type);
            Some((format!("map_key:{}", key), Some(format!("map_value:{}", val))))
        }
        _ => None,
    }
}

/// Check field/variable type for std containers and custom template usage,
/// creating appropriate edges with dual-mode target resolution.
fn extract_container_edges(type_name: &str, from_idx: usize, symbols: &[Symbol], edges: &mut Vec<(usize, usize, String)>) {
    // 1. Check for std::container<T> patterns
    if type_name.starts_with("std::") || type_name.contains("<") {
        // Extract inner type (between < and >)
        let inner = if let (Some(open), Some(close)) = (type_name.find('<'), type_name.rfind('>')) {
            &type_name[open+1..close]
        } else { return; };
        if let Some((edge1, edge2_opt)) = std_container_edge(type_name, inner) {
            // Resolve target per edge type name (not whole inner string)
            // e.g. map<int,User> → map_key:int and map_value:User resolved separately
            let t1 = edge1.find(':').map(|i| &edge1[i+1..]).unwrap_or("");
            let target1 = resolve_or_max(t1, symbols);
            edges.push((from_idx, target1, edge1));
            if let Some(edge2) = edge2_opt {
                let t2 = edge2.find(':').map(|i| &edge2[i+1..]).unwrap_or("");
                let target2 = resolve_or_max(t2, symbols);
                edges.push((from_idx, target2, edge2));
            }
            return; // std container handled, skip custom template check
        }
    }

    // 2. Check for custom template usage: MyVector<int>
    if let Some(lt) = type_name.find('<') {
        let template_name = type_name[..lt].trim();
        // Check if this is a known template_class/template_function
        let target = resolve_or_max(template_name, symbols);
        if target != usize::MAX {
            let usage_label = format!("template_use:{}", type_name);
            edges.push((from_idx, target, usage_label));
        }
    }
}

fn extract_template_params(source: &str, node: &Node, sym_idx: usize, edges: &mut Vec<(usize, usize, String)>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "template_parameter_list" { continue; }
        let mut pc = child.walk();
        for param in child.children(&mut pc) {
            let pkind = match param.kind() {
                "type_parameter_declaration" => "type",
                "parameter_declaration" => "non_type",
                _ => continue,
            };
            // For type_parameter_declaration, name is in type_identifier child
            // For parameter_declaration, name is in declarator
            let pname = if param.kind() == "type_parameter_declaration" {
                // Look for type_identifier child (e.g., "T" in "typename T")
                let mut found = None;
                let mut tc = param.walk();
                for c in param.children(&mut tc) {
                    if c.kind() == "type_identifier" {
                        found = c.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                        break;
                    }
                }
                found
            } else {
                param.child_by_field_name("declarator")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok().map(|s| s.to_string()))
            }.unwrap_or_else(|| "?".to_string());
            edges.push((sym_idx, usize::MAX, format!("template_param:{}[{}]", pname, pkind)));
        }
    }
}

fn extract_func_template(source: &str, node: &Node, file: &FileInfo, repo: &str,
    parent_class: Option<&str>, symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
    template_comment: &str,
) -> usize {
    let kind = if parent_class.is_some() { "method" } else { "template_function" };
    extract_func_impl(source, node, file, repo, parent_class, symbols, edges, kind, template_comment)
}

fn extract_class_template(source: &str, node: &Node, file: &FileInfo, repo: &str,
    symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
    template_comment: &str,
) -> usize {
    let kind = if node.kind() == "struct_specifier" { "template_struct" } else { "template_class" };
    extract_class_impl(source, node, file, repo, symbols, edges, kind, template_comment)
}


fn extract_func_impl(source: &str, node: &Node, file: &FileInfo, repo: &str,
    parent_class: Option<&str>, symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
    kind_override: &str, template_comment: &str,
) -> usize {
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
    let sig = if let (Some(ty), Some(decl)) = (node.child_by_field_name("type"), node.child_by_field_name("declarator")) {
        let ty_text = ty.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string();
        let decl_text = decl.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string();
        if ty_text.is_empty() { decl_text }
        else { format!("{} {}", ty_text, decl_text) }
    } else { full.clone() };
    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let mut comment = if template_comment.is_empty() { collect_comments(source, node) } else { template_comment.to_string() };
    let body_comments = collect_body_comments(source, node);
    if !body_comments.is_empty() {
        if !comment.is_empty() { comment.push_str(" | "); }
        comment.push_str(&body_comments);
    }
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: full.clone(),
        kind: kind_override.into(),
        content_hash: dedup::hash_content(&full),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1, 
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: Some(sig), parent_class: parent_class.map(|s| s.into()), namespace: None,
    doc_comment: comment,
    });
    let idx = symbols.len() - 1;
    if let Some(pc) = parent_class {
        emit_contains(symbols, pc, idx, &name, edges);
    }
    extract_calls_with_types(source, node, idx, edges, parent_class, symbols);
    extract_type_edges(source, node, idx, edges);
    idx
}

fn extract_class_impl(source: &str, node: &Node, file: &FileInfo, repo: &str,
    symbols: &mut Vec<Symbol>, edges: &mut Vec<(usize, usize, String)>,
    kind_override: &str, template_comment: &str,
) -> usize {
    let name = node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("anonymous");
    let mut def = extract_text(source, node.start_position().row as u32+1, node.end_position().row as u32+1);
    let mut comment = if template_comment.is_empty() { collect_comments(source, node) } else { template_comment.to_string() };
    let body_comments = collect_body_comments(source, node);
    if !body_comments.is_empty() {
        if !comment.is_empty() { comment.push_str(" | "); }
        comment.push_str(&body_comments);
    }
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: name.into(), kind: kind_override.into(),
        content_hash: dedup::hash_content(&name),
        file_path: file.path.clone(), line_start: node.start_position().row as u32+1,
        line_end: node.end_position().row as u32+1, language: Some("cpp".into()),
        signature: Some(name.into()), parent_class: None, namespace: None,
    doc_comment: comment,
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
    class_idx
}
// ── Macro extraction ──────────────────────────────────────────────────

fn is_noise_macro(name: &str) -> bool {
    let n = name.trim();
    if n.is_empty() { return true; }
    if n.ends_with("_H") || n.ends_with("_H_") || n.contains("INCLUDED") || n.contains("_h_") { return true; }
    if n.starts_with("__") { return true; }
    if matches!(n, "NDEBUG" | "_WIN32" | "_MSC_VER" | "_GLIBCXX_" | "LEVELDB_EXPORT") { return true; }
    false
}

fn extract_macro(source: &str, node: &Node, file: &FileInfo, repo: &str, symbols: &mut Vec<Symbol>) {
    let name = node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok()).unwrap_or("");
    if is_noise_macro(name) { return; }
    let mut def = extract_text(source, node.start_position().row as u32 + 1, node.end_position().row as u32 + 1);
    let mut comment = collect_comments(source, node);
    let body_comments = collect_body_comments(source, node);
    if !body_comments.is_empty() {
        if !comment.is_empty() { comment.push_str(" | "); }
        comment.push_str(&body_comments);
    }
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: name.into(), kind: "macro".into(),
        content_hash: dedup::hash_content(name),
        file_path: file.path.clone(), line_start: node.start_position().row as u32 + 1,
        line_end: node.end_position().row as u32 + 1, language: Some("cpp".into()),
        signature: Some(def), parent_class: None, namespace: None, doc_comment: comment,
    });
}

// ── String literal extraction ────────────────────────────────────────

fn strip_string_delimiters(s: &str) -> &str {
    let s = s.trim();
    // "..." or u8"..." etc
    if s.len() >= 2 && s.ends_with('"') {
        let start = s.find('"').unwrap_or(0);
        return &s[start+1..s.len()-1];
    }
    // R"(...)" or R"foo(...)foo"
    if s.starts_with("R\"") {
        if let Some(open) = s.find('(') {
            if let Some(close) = s.rfind(')') {
                return &s[open+1..close];
            }
        }
    }
    // <...>
    if s.len() >= 2 && s.starts_with('<') && s.ends_with('>') {
        return &s[1..s.len()-1];
    }
    s
}

fn collect_all_string_literals(source: &str, node: &Node, file: &FileInfo, repo: &str, symbols: &mut Vec<Symbol>) {
    let kind = node.kind();
    if kind == "string_literal" || kind == "raw_string_literal" || kind == "concatenated_string" || kind == "system_lib_string" {
        extract_string_literal(source, node, file, repo, symbols);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_all_string_literals(source, &child, file, repo, symbols);
    }
}

fn extract_string_literal(source: &str, node: &Node, file: &FileInfo, repo: &str, symbols: &mut Vec<Symbol>) {
    let raw = node.utf8_text(source.as_bytes()).ok().unwrap_or("");
    let name = strip_string_delimiters(raw);
    if name.is_empty() { return; }
    let mut def = extract_text(source, node.start_position().row as u32 + 1, node.end_position().row as u32 + 1);
    let comment = collect_comments(source, node);
    if !comment.is_empty() { def = format!("{}\n{}", comment, def); }
    symbols.push(Symbol {
        id: None, repo: repo.into(), name: name.into(), kind: "string_literal".into(),
        content_hash: dedup::hash_content(name),
        file_path: file.path.clone(), line_start: node.start_position().row as u32 + 1,
        line_end: node.end_position().row as u32 + 1, language: Some("cpp".into()),
        signature: Some(def), parent_class: None, namespace: None, doc_comment: comment,
    });
}

