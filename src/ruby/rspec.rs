use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RSpecEvent {
    Start {
        count: Option<i64>,
    },
    ExampleStarted {
        id: String,
        location: String,
        file_path: PathBuf,
        description: Option<String>,
    },
    ExamplePassed {
        id: String,
        load_time: Option<f64>,
        location: String,
        file_path: PathBuf,
        description: String,
        run_time: f64,
    },
    ExampleFailed {
        id: String,
        load_time: Option<f64>,
        location: Option<String>,
        file_path: PathBuf,
        description: Option<String>,
        run_time: f64,
        exception: Option<String>,
    },
    Stop {},
    Error {
        msg: String,
    },
    Exit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSpecConfiguration {
    pub path_to_rspec: String,
    pub use_bundler: bool,
    pub env: HashMap<String, String>,
}

impl Default for RSpecConfiguration {
    fn default() -> Self {
        RSpecConfiguration {
            path_to_rspec: String::from("rspec"),
            use_bundler: false,
            env: HashMap::new(),
        }
    }
}

pub struct RSpecRun {
    handle: tokio::task::JoinHandle<()>,
    cmd: std::process::Child,
}

impl RSpecRun {
    pub async fn wait(self) -> anyhow::Result<()> {
        self.handle
            .await
            .map_err(|_e| anyhow::Error::msg("rspec wait error"))
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.cmd.kill().map_err(anyhow::Error::from)
    }
}

pub struct RSpec {
    config: RSpecConfiguration,
}

impl RSpec {
    pub fn new(config: RSpecConfiguration) -> Self {
        RSpec { config }
    }

    pub fn run<T: AsRef<str>>(
        &self,
        locations: Vec<T>,
        tx: Sender<RSpecEvent>,
    ) -> anyhow::Result<RSpecRun> {
        let ref_locations: Vec<&str> = locations.iter().map(|t| t.as_ref()).collect();
        let config = &self.config.clone();
        let use_bundler = config.use_bundler;

        let program = match use_bundler {
            true => "bundle",
            false => &config.path_to_rspec,
        };

        let mut args: Vec<&str> = Vec::new();

        if use_bundler {
            args.push("exec");
            args.push(&config.path_to_rspec);
        }

        args.push("--format");
        args.push("RustRspecFormatter");
        args.push("--require");
        args.push("./test/rust_rspec_formatter.rb");

        let args_with_locations: Vec<&&str> = args.iter().chain(ref_locations.iter()).collect();
        dbg!(program);
        dbg!(args_with_locations.clone());

        let mut cmd = Command::new(program)
            .args(args_with_locations)
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = cmd.stdout.take().unwrap();
        #[allow(unused_must_use)]
        let handle = tokio::spawn(async move {
            let mut stdout_reader = BufReader::with_capacity(10, stdout);

            loop {
                let mut buf = String::new();
                let line = stdout_reader.read_line(&mut buf);

                match line {
                    Err(err) => {
                        tx.send(RSpecEvent::Error {
                            msg: err.to_string(),
                        })
                        .await;
                        break;
                    }
                    Ok(usize) if usize == 0 => {
                        break;
                    }
                    Ok(_usize) => {
                        let deser = serde_json::from_str::<RSpecEvent>(&buf);

                        if deser.is_err() {
                            let err = deser.err().unwrap();
                            tx.send(RSpecEvent::Error {
                                msg: err.to_string(),
                            })
                            .await;
                            break;
                        }

                        let event = deser.unwrap();
                        let res = tx.send(event).await;

                        if res.is_err() {
                            break;
                        }
                    }
                }
            }

            tx.send(RSpecEvent::Exit).await;
            drop(tx);
        });

        Ok(RSpecRun { handle, cmd })
    }
}
