use regex::Regex;
use std::{collections::HashMap, env, fs};
use tokio::{process, task::JoinSet};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let mut procfile = Procfile::new(env::args().nth(1).expect("No procfile path specified"));
    procfile.parse()?;

    let mut handles = JoinSet::new();

    for (_, cmd) in procfile.commands.iter() {
        let cmd = cmd.clone();
        handles.spawn(async move {
            process::Command::new(cmd)
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
