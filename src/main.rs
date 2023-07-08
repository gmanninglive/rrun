use anyhow::Ok;
use regex::Regex;

use log::{Level, LevelFilter};
use std::{collections::HashMap, env, fs, sync::Arc};
use tokio::{process, sync::Mutex, task::JoinSet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pids: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .init();

    let mut procfile = Procfile::new(env::args().nth(1).expect("No procfile path specified"));
    procfile.parse()?;

    let mut handles = JoinSet::new();

    for (p_name, cmd) in procfile.commands.iter() {
        let split_cmd = cmd.split_ascii_whitespace();
        let mut cmd: String = String::new();
        let mut args: Vec<String> = Vec::new();

        for (i, str) in split_cmd.enumerate() {
            if i == 0 {
                cmd = str.into();
            } else {
                args.push(str.into());
            }
        }

        if cmd.is_empty() {
            continue;
        }

        handles.spawn({
            let p_name = p_name.clone();
            let pids = pids.clone();

            async move {
                let mut child = process::Command::new(cmd)
                    .current_dir(env::current_dir().unwrap())
                    .args(args)
                    .spawn()
                    .expect("err: ");

                pids.lock().await.push(
                    child
                        .id()
                        .expect(format!("error getting pid for {p_name}").as_str()),
                );

                child
                    .wait()
                    .await
                    .expect("child process encountered an error");
            }
        });
    }

    // write pids to log file
    fs::write("~/.rman.pids", format!("{:?}", pids.lock().await))?;

    loop {
        tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            log::log!(Level::Info, "attempting graceful shutdown {} processes", handles.len());
            handles.shutdown().await;
            break;
        }
        }
    }

    Ok(())
}

struct Procfile {
    commands: HashMap<String, String>,
    path: String,
}

impl Procfile {
    fn new(path: String) -> Self {
        Procfile {
            commands: HashMap::new(),
            path,
        }
    }

    fn parse(&mut self) -> anyhow::Result<&Self> {
        let reg = Regex::new(r"(?m)^(?P<key>[A-Za-z0-9_]+):\s*(?P<cmd>.+)$")
            .expect("Failed building regex");

        let file = fs::read_to_string(&self.path)?;

        let matches = reg.captures_iter(file.as_str());

        for cap in matches {
            self.commands.insert(
                cap.name("key").expect("error parsing key").as_str().into(),
                cap.name("cmd").expect("error parsing cmd").as_str().into(),
            );
        }

        Ok(self)
    }
}
