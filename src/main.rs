#![allow(dead_code, unused_variables)]
mod cli; mod config; mod doc; mod embedding; mod ignore; mod indexer; mod linking;
mod mcp; mod query; mod storage; mod util;

extern "C" {
    /// Register sqlite-vec extension via sqlite3_auto_extension.
    /// Called once at startup, before any database connection is opened.
    fn vec0_static_init();
}
use clap::Parser;
#[allow(unused_imports)]
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

    // Register sqlite-vec extension (statically compiled, no .so needed)
    // Must be called before any database connection is opened.
    unsafe { vec0_static_init(); }

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
        None => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            cmd.print_help()?;
            println!();
        }
    }
    Ok(())
}
