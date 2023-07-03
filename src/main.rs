mod log;
use regex::Regex;
use std::{collections::HashMap, env, fs};
use tokio::{process, task::JoinSet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut procfile = Procfile::new(env::args().nth(1).expect("No procfile path specified"));
    procfile.parse()?;

    let mut handles = JoinSet::new();

    for (_, cmd) in procfile.commands.iter() {
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

        handles.spawn(async move {
            process::Command::new(cmd)
                .current_dir(env::current_dir().unwrap())
                .args(args)
                .spawn()
                .expect("err: ")
                .wait()
                .await
        });
    }

    loop {
        tokio::select! {
        _ = handles.join_next() => {
            handles.abort_all();
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
