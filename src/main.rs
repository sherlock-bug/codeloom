#![allow(dead_code, unused_variables)]
mod cli; mod config; mod doc; mod embedding; mod ignore; mod indexer; mod linking;
mod mcp; mod query; mod storage;
use clap::Parser;
use clap::CommandFactory;
use clap_complete::{generate, shells};

#[derive(Parser)]
#[command(
    name = "codeloom",
    version,
    about = "团队代码知识管理工具 — 为 LLM Agent 编织代码库知识图谱",
    long_about = "CodeLoom 把零散的代码、文档、业务知识编织成一张可查询的知识图谱，

让 OpenCode/Claude Code 等 AI 编码助手中的 LLM 能理解百万行级别的多代码仓项目。

纯本地运行，零外部 API 依赖，代码不出内网。",
    after_help = "示例:
  codeloom index .                        # 索引当前目录
  codeloom status --repo myrepo           # 查看状态
  codeloom mcp                            # 启动 MCP 服务
  codeloom branch set-alias 23B release/2023-B --repo myrepo

Tab 补全: source <(codeloom completion bash)

项目: https://github.com/sherlock-bug/codeloom"
)]
struct Cli { #[command(subcommand)] command: Option<cli::Command> }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Some(cli::Command::Completion { shell }) => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            let name = cmd.get_name().to_string();
            match shell.as_str() {
                "bash" => generate(shells::Bash, &mut cmd, &name, &mut std::io::stdout()),
                "zsh" => generate(shells::Zsh, &mut cmd, &name, &mut std::io::stdout()),
                "fish" => generate(shells::Fish, &mut cmd, &name, &mut std::io::stdout()),
                "powershell" => generate(shells::PowerShell, &mut cmd, &name, &mut std::io::stdout()),
                s => { eprintln!("Unknown shell: {}. Supported: bash, zsh, fish, powershell", s); }
            }
        }
        Some(cmd) => cli::run(cmd).await?,
        None => mcp::serve().await?,
    }
    Ok(())
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
