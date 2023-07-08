use std::fs;

use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub cmds: Vec<Cmd>,
    pub path: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cmd {
    pub name: String,
    pub cmd: String,
    pub args: Vec<String>,
    pub stdin: RmanStdio,
    pub stdout: RmanStdio,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RmanStdio {
    Inherit,
    Pipe,
    Null,
}

impl Config {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            cmds: Vec::new(),
            path: path.into(),
        }
    }

    pub fn push_cmd(&mut self, cmd: Cmd) -> &mut Self {
        self.cmds.push(cmd);
        self
    }
}

pub struct Procfile {
    path: String,
}

impl Procfile {
    pub fn new(path: impl Into<String>) -> Self {
        Procfile { path: path.into() }
    }

    pub fn parse(&mut self) -> anyhow::Result<Config> {
        let mut config = Config::new(&self.path);

        let reg = Regex::new(r"(?m)^(?P<key>[A-Za-z0-9_]+):\s*(?P<cmd>.+)$")
            .expect("Failed building regex");

        let file = fs::read_to_string(&self.path)?;

        let matches = reg.captures_iter(file.as_str());

        for cap in matches {
            let cmd: String = cap.name("cmd").expect("error parsing cmd").as_str().into();

            if cmd.is_empty() {
                continue;
            }

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

            config.push_cmd(Cmd {
                name: cap.name("key").expect("error parsing key").as_str().into(),
                cmd,
                args,
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Inherit,
            });
        }

        Ok(config)
    }
}

pub mod test {
    use super::*;

    #[test]
    fn test_constructor() {
        let mut config = Config::new("./config.yml");

        let cmd = Cmd {
            name: "test".to_string(),
            cmd: "ls".to_string(),
            args: Vec::new(),
            stdin: RmanStdio::Pipe,
            stdout: RmanStdio::Null,
        };

        config.push_cmd(cmd.clone());

        assert_eq!(
            config,
            Config {
                cmds: vec![cmd],
                path: "./config.yml".to_string()
            }
        );
    }
}
