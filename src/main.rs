mod config;

use anyhow::Ok;
use config::{Cmd, Config};

use log::{Level, LevelFilter};
use std::{env, fs, process::Stdio, sync::Arc};
use tokio::{process, sync::Mutex, task::JoinSet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pids: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .init();

    let mut config = Config::new(env::args().nth(1).expect("No config file path specified"));
    config.parse()?;

    let mut handles = JoinSet::new();

    for command in config.cmds.iter() {
        handles.spawn({
            let Cmd {
                name,
                cmd,
                args,
                stdin,
                stdout,
            } = command.clone();
            let pids = pids.clone();

            async move {
                let mut child = process::Command::new(cmd)
                    .current_dir(env::current_dir().unwrap())
                    .args(args.to_owned())
                    .stdin(match stdin {
                        Some(config::RmanStdio::Inherit) => Stdio::inherit(),
                        Some(config::RmanStdio::Pipe) => Stdio::piped(),
                        Some(config::RmanStdio::Null) => Stdio::null(),
                        None => Stdio::inherit(),
                    })
                    .stdout(match stdout {
                        Some(config::RmanStdio::Inherit) => Stdio::inherit(),
                        Some(config::RmanStdio::Pipe) => Stdio::piped(),
                        Some(config::RmanStdio::Null) => Stdio::null(),
                        None => Stdio::inherit(),
                    })
                    .spawn()
                    .expect("err: ");

                pids.lock().await.push(
                    child
                        .id()
                        .expect(format!("error getting pid for {name}").as_str()),
                );

                child
                    .wait()
                    .await
                    .expect("child process encountered an error");
            }
        });
    }

    loop {
        tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            let home = std::env::var("HOME").expect("$HOME env missing");
            // write pids to log file
            fs::write(
                format!("{home}/.rman.pids"),
                format!("{:?}\n", pids.lock().await),
            )?;

            log::log!(Level::Info, "attempting graceful shutdown of {} processes", handles.len());
            handles.shutdown().await;
            break;
        }
        }
    }

    Ok(())
}
