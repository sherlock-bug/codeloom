#![allow(dead_code, unused_variables)]
mod cli; mod config; mod doc; mod embedding; mod ignore; mod indexer; mod linking;
mod mcp; mod query; mod storage;
use clap::Parser;

#[derive(Parser)]
#[command(name = "codeloom", version, about = "团队代码知识管理工具")]
struct Cli { #[command(subcommand)] command: Option<cli::Command> }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    match Cli::parse().command {
        Some(cmd) => cli::run(cmd).await,
        None => mcp::serve().await,
    }
}
#[test]
fn dump_template_method_ast() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).unwrap();
    let src = include_str!("/tmp/test_create.cpp");
    let tree = parser.parse(src, None).unwrap();
    let root = tree.root_node();
    fn dump(node: tree_sitter::Node, src: &str, depth: usize) {
        let kind = node.kind();
        let text = node.utf8_text(src.as_bytes()).unwrap_or("");
        let text = if text.len() > 60 { format!("{}...", &text[..57]) } else { text.to_string() };
        println!("{:indent$}{} \"{}\"", " ", kind, text, indent=depth*2);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            dump(child, src, depth+1);
        }
    }
    dump(root, src, 0);
}
