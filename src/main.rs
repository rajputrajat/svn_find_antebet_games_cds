use anyhow::Result;
use log::info;
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

async fn get_svn_list(path: &str) -> Result<SvnList> {
    let svn_cmd = SvnCmd::new(
        Credentials {
            username: "svc-p-blsrobo".to_owned(),
            password: "Comewel@12345".to_owned(),
        },
        None,
    )?;
    Ok(svn_cmd.list(path, true).await?)
}

fn parse_cds_config_and_check_lines(cds_config_path: &str) -> Result<()> {
    Ok(())
}
