#!/usr/bin/env python3
import re

path = '/mnt/d/RagMcpHermes/codeloom/src/indexer/clang/ast.rs'
with open(path, 'r') as f:
    content = f.read()

# Remove VMETH debug block - replace with clean version
old_vmeth = '''                    // Record virtual methods for override detection
                    if is_virtual && kind == "CXXMethodDecl" {
                        eprintln!(
                            "EDEBUG-VMETH: is_virtual={} kind={} name={:?} parent={:?}",
                            is_virtual, kind, n, parent_class
                        );
                        if let Some(pc) = parent_class {
                            self.class_virtual_methods
                                .entry(pc.to_string())
                                .or_default()
                                .push(n.to_string());
                        } else {
                            eprintln!("EDEBUG-VMETH: RECORD SKIPPED — no parent_class");
                        }
                    } else {
                        if kind == "CXXMethodDecl" {
                            eprintln!("EDEBUG-VMETH: NOT recorded — is_virtual={}", is_virtual);
                        }
                    }'''

new_vmeth = '''                    // Record virtual methods for override detection
                    if is_virtual && kind == "CXXMethodDecl" {
                        if let Some(pc) = parent_class {
                            self.class_virtual_methods
                                .entry(pc.to_string())
                                .or_default()
                                .push(n.to_string());
                        }
                    }'''

content = content.replace(old_vmeth, new_vmeth)

# Remove OVERRIDE debug lines - the while loop content
old_override_eprintln1 = '''                        eprintln!(
                            "EDEBUG-OVERRIDE: checking {} (parent={}, bases={:?}, vmethods={:?})",
                            &qname, pc, &work, self.class_virtual_methods
                        );
                        while let Some(base) = work.pop() {
                            if visited.contains(&base) {
                                continue;
                            }
                            visited.push(base.clone());
                            if let Some(vmethods) = self.class_virtual_methods.get(&base) {
                                eprintln!(
                                    "EDEBUG-OVERRIDE: base={} vmethods={:?} looking for {}",
                                    base, vmethods, method_name
                                );
                                if vmethods.contains(&method_name) {
                                    eprintln!(
                                        "EDEBUG-OVERRIDE: FOUND! emitting overrides:{}::{}",
                                        base, method_name
                                    );
                                    self.result.edges.push(Edge {
                                        source_name: qname.clone(),
                                        source_ns: ns.clone(),
                                        target_name: format!("{}::{}", base, method_name),
                                        edge_type: "overrides".to_string(),
                                    });
                                }
                            } else {
                                eprintln!("EDEBUG-OVERRIDE: base={} has NO vmethods entry", base);
                            }'''

new_override = '''                        while let Some(base) = work.pop() {
                            if visited.contains(&base) {
                                continue;
                            }
                            visited.push(base.clone());
                            if let Some(vmethods) = self.class_virtual_methods.get(&base) {
                                if vmethods.contains(&method_name) {
                                    self.result.edges.push(Edge {
                                        source_name: qname.clone(),
                                        source_ns: ns.clone(),
                                        target_name: format!("{}::{}", base, method_name),
                                        edge_type: "overrides".to_string(),
                                    });
                                }
                            }'''

content = content.replace(old_override_eprintln1, new_override)

with open(path, 'w') as f:
    f.write(content)

print("Debug lines cleaned")
