use std::{fs, path::Path};

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    pub cmds: Vec<Cmd>,
    #[serde(skip)]
    pub path: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Cmd {
    pub name: String,
    pub cmd: String,
    pub args: Vec<String>,

    #[serde(default)]
    pub stdin: RmanStdio,
    #[serde(default)]
    pub stdout: RmanStdio,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RmanStdio {
    #[default]
    Inherit,
    Pipe,
    Null,
    File(String),
}

impl Config {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            cmds: Vec::new(),
            path: path.into(),
        }
    }

    pub fn init() -> anyhow::Result<Self> {
        let clap = clap::Command::new("rman")
            .arg(
                clap::Arg::new("config_path")
                    .index(1)
                    .help(
                        r#"Path to config file, this can be json, yaml or a procfile. 
Note using a procfile disables the ability to set stdio per command"#,
                    )
                    .action(clap::ArgAction::Set)
                    .value_name("config_path"),
            )
            .get_matches();

        let path = clap
            .get_one::<String>("config_path")
            .expect("No config file path specified");

        let mut config = Config::new(path);
        config.parse()?;

        Ok(config)
    }

    pub fn push_cmd(&mut self, cmd: Cmd) -> &mut Self {
        self.cmds.push(cmd);
        self
    }

    pub fn parse(&mut self) -> Result<&Self, anyhow::Error> {
        let file = fs::read_to_string(&self.path).expect("error reading config file");

        if let Some(ext) = Path::new(&self.path).extension() {
            if ext == "yml" || ext == "yaml" {
                self.cmds = serde_yaml::from_str::<Vec<Cmd>>(file.as_str())?;
            };
            if ext == "json" {
                self.cmds = serde_json::from_str(file.as_str())?;
            }
        } else {
            self.cmds = parse_procfile(file)?;
        }

        Ok(self)
    }
}

pub fn parse_procfile(file: String) -> anyhow::Result<Vec<Cmd>> {
    let mut cmds: Vec<Cmd> = Vec::new();

    let reg = Regex::new(
        r"(?m)^(?P<NAME>[A-Za-z0-9_]+):(?P<STDIN>\sstdin>(inherit|null|file=\S+))?(?P<STDOUT>\sstdout>(inherit|null|file=\S+))?\s*(?P<CMD>.+)$")
    .expect("Failed building regex");

    let matches = reg.captures_iter(file.as_str());

    for cap in matches {
        let cmd: String = cap.name("CMD").expect("error parsing cmd").as_str().into();
        let stdin = match cap.name("STDIN") {
            Some(s) => {
                let s = s.as_str().replace("stdin>", "");
                let s = s.trim();

                match s {
                    "null" => RmanStdio::Null,
                    "inherit" => RmanStdio::Inherit,
                    _ if s.starts_with("file=") => RmanStdio::File(
                        s.split("file=")
                            .last()
                            .expect("no file path specified for stdin")
                            .to_string(),
                    ),
                    _ => RmanStdio::Inherit,
                }
            }
            None => RmanStdio::Inherit,
        };
        let stdout = match cap.name("STDOUT") {
            Some(s) => {
                let s = s.as_str().replace("stdout>", "");
                let s = s.trim();

                match s {
                    "null" => RmanStdio::Null,
                    "inherit" => RmanStdio::Inherit,
                    _ if s.starts_with("file=") => RmanStdio::File(
                        s.split("file=")
                            .last()
                            .expect("no file path specified for stdin")
                            .to_string(),
                    ),
                    _ => RmanStdio::Inherit,
                }
            }
            None => RmanStdio::Inherit,
        };

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

        cmds.push(Cmd {
            name: cap.name("NAME").expect("error parsing key").as_str().into(),
            cmd,
            args,
            stdin,
            stdout,
        });
    }

    Ok(cmds)
}

#[cfg(test)]
pub mod test {
    #[allow(unused_imports)]
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

    #[test]
    fn test_deserialize_yaml() {
        let mut config = Config::new("test/fixtures/config.yml");
        let _ = config.parse();

        assert_eq!(
            config,
            Config {
                path: "test/fixtures/config.yml".to_string(),
                cmds: vec![
                    Cmd {
                        name: "web_1".to_string(),
                        cmd: "cargo".to_string(),
                        args: vec![
                            "run".to_string(),
                            "--example".to_string(),
                            "echo".to_string()
                        ],
                        stdin: RmanStdio::Inherit,
                        stdout: RmanStdio::Inherit
                    },
                    Cmd {
                        name: "web_2".to_string(),
                        cmd: "cargo".to_string(),
                        args: vec![
                            "run".to_string(),
                            "--example".to_string(),
                            "echo".to_string()
                        ],
                        stdin: RmanStdio::Null,
                        stdout: RmanStdio::Null
                    }
                ]
            }
        )
    }

    #[test]
    fn test_deserialize_json() {
        let mut config = Config::new("test/fixtures/config.json");
        let _ = config.parse();

        assert_eq!(
            config,
            Config {
                path: "test/fixtures/config.json".to_string(),
                cmds: vec![
                    Cmd {
                        name: "web_1".to_string(),
                        cmd: "cargo".to_string(),
                        args: vec![
                            "run".to_string(),
                            "--example".to_string(),
                            "echo".to_string()
                        ],
                        stdin: RmanStdio::Inherit,
                        stdout: RmanStdio::Inherit
                    },
                    Cmd {
                        name: "web_2".to_string(),
                        cmd: "cargo".to_string(),
                        args: vec![
                            "run".to_string(),
                            "--example".to_string(),
                            "echo".to_string()
                        ],
                        stdin: RmanStdio::Null,
                        stdout: RmanStdio::Null
                    }
                ]
            }
        )
    }

    #[test]
    fn test_deserialize_procfile() {
        let mut config = Config::new("test/fixtures/procfile");
        let _ = config.parse();

        assert_eq!(
            config,
            Config {
                path: "test/fixtures/procfile".to_string(),
                cmds: vec![
                    Cmd {
                        name: "web_1".to_string(),
                        cmd: "cargo".to_string(),
                        args: vec![
                            "run".to_string(),
                            "--example".to_string(),
                            "echo".to_string(),
                            "--".to_string(),
                            "8080".to_string()
                        ],
                        stdin: RmanStdio::Inherit,
                        stdout: RmanStdio::Inherit
                    },
                    Cmd {
                        name: "web_2".to_string(),
                        cmd: "cargo".to_string(),
                        args: vec![
                            "run".to_string(),
                            "--example".to_string(),
                            "echo".to_string(),
                            "--".to_string(),
                            "8081".to_string()
                        ],
                        stdin: RmanStdio::Inherit,
                        stdout: RmanStdio::Inherit
                    }
                ]
            }
        )
    }

    #[test]
    fn test_parse_procfile() {
        let procfile_stdio_defined =
            "cmd_1: stdin>null stdout>inherit ls -a\ncmd_2: stdin>inherit stdout>null ls -a"
                .to_string();

        let result = parse_procfile(procfile_stdio_defined).unwrap();
        let expected = vec![
            Cmd {
                name: "cmd_1".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Null,
                stdout: RmanStdio::Inherit,
            },
            Cmd {
                name: "cmd_2".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Null,
            },
        ];
        assert_eq!(result, expected);

        let procfile_stdin_defined =
            "cmd_1: stdin>null ls -a\ncmd_2: stdin>inherit ls -a".to_string();

        let result = parse_procfile(procfile_stdin_defined).unwrap();
        let expected = vec![
            Cmd {
                name: "cmd_1".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Null,
                stdout: RmanStdio::Inherit,
            },
            Cmd {
                name: "cmd_2".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Inherit,
            },
        ];
        assert_eq!(result, expected);

        let procfile_stdout_defined =
            "cmd_1: stdout>null ls -a\ncmd_2: stdout>null ls -a".to_string();

        let result = parse_procfile(procfile_stdout_defined).unwrap();
        let expected = vec![
            Cmd {
                name: "cmd_1".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Null,
            },
            Cmd {
                name: "cmd_2".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Null,
            },
        ];
        assert_eq!(result, expected);

        let procfile_stdio_file =
            "cmd_1: stdin>file=test_in.txt stdout>file=test_out.txt ls -a\ncmd_2: ls -a"
                .to_string();

        let result = parse_procfile(procfile_stdio_file).unwrap();
        let expected = vec![
            Cmd {
                name: "cmd_1".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::File("test_in.txt".to_string()),
                stdout: RmanStdio::File("test_out.txt".to_string()),
            },
            Cmd {
                name: "cmd_2".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Inherit,
            },
        ];
        assert_eq!(result, expected);

        let procfile_stdio_not_defined = "cmd_1: ls -a\ncmd_2: ls -a".to_string();

        let result = parse_procfile(procfile_stdio_not_defined).unwrap();
        let expected = vec![
            Cmd {
                name: "cmd_1".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Inherit,
            },
            Cmd {
                name: "cmd_2".to_string(),
                cmd: "ls".to_string(),
                args: vec!["-a".to_string()],
                stdin: RmanStdio::Inherit,
                stdout: RmanStdio::Inherit,
            },
        ];
        assert_eq!(result, expected);
    }
}
