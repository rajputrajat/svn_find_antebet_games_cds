use anyhow::Result;
use async_std::{fs::read_to_string, task};
use log::trace;
use serde::Deserialize;
use std::{env, process::exit, time::Instant};
use strip_bom::*;
use svn_cmd::{Credentials, SvnCmd, SvnList};

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();
    let start_instant = Instant::now();
    let cmd_ops = get_cmd_args();
    process(&cmd_ops).await?;
    println!(
        "cmd executed in {} msecs",
        start_instant.elapsed().as_millis()
    );
    Ok(())
}

const ERROR_HELP: &str = "##
USAGE:
    ./svn_find_antebet_games_cds.exe --svn-path <svn url>
    ./svn_find_antebet_games_cds.exe --svn-path <svn url> --list-file <svn list --xml svn-path>
##";

enum CmdOptions {
    SvnPath(String),
    ListFilePath(String, String),
}

fn get_cmd_args() -> CmdOptions {
    if let Some(flag1) = env::args().nth(1) {
        if &flag1 == "--svn-path" {
            if let Some(url) = env::args().nth(2) {
                if let Some(flag2) = env::args().nth(3) {
                    if &flag2 == "--list-file" {
                        if let Some(path) = env::args().nth(4) {
                            return CmdOptions::ListFilePath(url, path);
                        }
                    }
                } else {
                    return CmdOptions::SvnPath(url);
                }
            }
        }
    }
    eprintln!("{}", ERROR_HELP);
    exit(1);
}

#[derive(Clone)]
struct SvnCommand {
    cmd: SvnCmd,
}

#[derive(Deserialize)]
struct GameConfiguration {
    #[serde(rename(deserialize = "LineOptions"))]
    line_options: LineOptions,
}

#[derive(Deserialize)]
struct LineOptions {
    #[serde(rename(deserialize = "LineOption"))]
    line_option: LineOption,
}

#[derive(Deserialize)]
struct LineOption {
    #[serde(rename(deserialize = "Line"))]
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

    async fn get_svn_list(&self, url: &str) -> Result<SvnList> {
        Ok(self.cmd.list(url, true).await?)
    }

    async fn get_svn_list_from_list_cmd_out(&self, path: &str) -> Result<SvnList> {
        let xml_str = read_to_string(&path).await?;
        Ok(self.cmd.list_from_svn_list_xml_output(&xml_str).await?)
    }

    async fn parse_cds_config_and_check_lineoptions_count(
        &self,
        cds_config_path: &str,
    ) -> Result<usize> {
        trace!("parsing config file: {}", cds_config_path);
        let xml_text = self.cmd.cat(cds_config_path).await?;
        let xml_text = xml_text.strip_bom();
        let config: GameConfiguration = serde_xml_rs::from_str(xml_text)?;
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

async fn process(cmd_ops: &CmdOptions) -> Result<()> {
    let cmd = SvnCommand::new()?;
    let (url, list) = match &cmd_ops {
        CmdOptions::SvnPath(url) => (url.clone(), cmd.get_svn_list(url).await?),
        CmdOptions::ListFilePath(url, path) => {
            (url.clone(), cmd.get_svn_list_from_list_cmd_out(path).await?)
        }
    };
    let cfg_files = find_cfg_file_paths(&url, list);
    let mut tasks = Vec::new();
    for cfg_file in cfg_files.into_iter() {
        let cmd = cmd.clone();
        let cfg_file = cfg_file.clone();
        tasks.push(task::spawn(async move {
            (
                cmd.parse_cds_config_and_check_lineoptions_count(&cfg_file)
                    .await
                    .unwrap(),
                cfg_file,
            )
        }));
    }
    task::block_on(async {
        for t in tasks {
            let (antebet_count, cfg_file) = t.await;
            trace!(
                "cfg file: {}, with antebet lines: {}",
                cfg_file,
                antebet_count
            );
            if antebet_count > 1 {
                println!(
                    "cfg file: {}, with antebet lines: {}",
                    cfg_file, antebet_count
                );
            }
        }
    });
    Ok(())
}
