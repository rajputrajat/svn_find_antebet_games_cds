use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::{env, process::exit, time::Instant};
use svn_cmd::{Credentials, SvnCmd, SvnList};

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let start_instant = Instant::now();
    let root_path = get_svn_path_from_cli_args();

    Ok(())
}

fn get_svn_path_from_cli_args() -> String {
    if let Some(path) = env::args().nth(1) {
        path
    } else {
        eprintln!("provide svn path as cli argument.");
        exit(1);
    }
}

struct SvnCommand {
    cmd: SvnCmd,
}

#[derive(Deserialize)]
struct GameConfiguration {
    line_options: LineOptions,
}

#[derive(Deserialize)]
struct LineOptions {
    line_option: LineOption,
}

#[derive(Deserialize)]
struct LineOption {
    line: Vec<Line>,
}

#[derive(Deserialize)]
struct Line {}

impl SvnCommand {
    fn new() -> Result<Self> {
        Ok(Self {
            cmd: SvnCmd::new(
                Credentials {
                    username: "svc-p-blsrobo".to_owned(),
                    password: "Comewel@12345".to_owned(),
                },
                None,
            )?,
        })
    }

    async fn get_svn_list(&self, path: &str) -> Result<SvnList> {
        Ok(self.cmd.list(path, true).await?)
    }

    async fn parse_cds_config_and_check_lineoptions_count(
        &self,
        cds_config_path: &str,
    ) -> Result<usize> {
        let xml_text = self.cmd.cat(cds_config_path).await?;
        let config: GameConfiguration = serde_xml_rs::from_str(&xml_text)?;
        Ok(config.line_options.line_option.line.len())
    }
}
