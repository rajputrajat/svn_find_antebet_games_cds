use anyhow::Result;
use async_std::task;
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

#[derive(Clone)]
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

const PATH_FILTERS: &[&str] = &["tags", "_signed", "cds_configuration.xml"];

fn find_cfg_file_paths(path: &str, svn_list: SvnList) -> Vec<String> {
    svn_list
        .iter()
        .filter_map(|e| {
            let full_path = format!("{}/{}", path, e.name);
            if PATH_FILTERS.iter().all(|f| full_path.contains(f)) {
                Some(full_path)
            } else {
                None
            }
        })
        .collect()
}

async fn process(path: &str) -> Result<()> {
    let cmd = SvnCommand::new()?;
    let list = cmd.get_svn_list(path).await?;
    let cfg_files = find_cfg_file_paths(path, list);
    let mut tasks = Vec::new();
    for cfg_file in cfg_files.into_iter() {
        let cmd = cmd.clone();
        tasks.push(task::spawn(async move {
            cmd.parse_cds_config_and_check_lineoptions_count(&cfg_file)
                .await
                .unwrap();
        }));
    }
    Ok(())
}
