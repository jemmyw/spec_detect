use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

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
        description: Option<String>,
    },
    ExamplePassed {
        id: String,
        load_time: Option<f64>,
        location: String,
        description: String,
        run_time: f64,
    },
    ExampleFailed {
        id: String,
        load_time: Option<f64>,
        location: Option<String>,
        description: Option<String>,
        run_time: f64,
        exception: Option<String>,
    },
    Stop {},
}

pub struct RSpecConfiguration {
    pub path_to_rspec: String,
    pub use_bundler: bool,
}

impl Default for RSpecConfiguration {
    fn default() -> Self {
        RSpecConfiguration {
            path_to_rspec: String::from("rspec"),
            use_bundler: false,
        }
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
        tx: std::sync::mpsc::Sender<Result<RSpecEvent, anyhow::Error>>,
    ) -> anyhow::Result<()> {
        let ref_locations: Vec<&str> = locations.iter().map(|t| t.as_ref()).collect();
        let config = &self.config;
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
        dbg!(args_with_locations.clone());

        let mut cmd = Command::new(program)
            .args(args_with_locations)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let mut stdout_reader = BufReader::with_capacity(10, stdout);

            loop {
                let mut buf = String::new();
                let line = stdout_reader.read_line(&mut buf);

                match line.map_err(anyhow::Error::from).and_then(|_u| {
                    let event =
                        serde_json::from_str::<RSpecEvent>(&buf).map_err(anyhow::Error::from);
                    tx.send(event).map_err(anyhow::Error::from)
                }) {
                    Ok(_) => {}
                    Err(_) => break,
                };
            }

            drop(tx);
        }

        cmd.wait()?;

        Ok(())
    }
}
