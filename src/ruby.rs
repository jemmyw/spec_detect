use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::Result;

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

    pub fn run<T: AsRef<str>>(&self, locations: Vec<T>) -> anyhow::Result<Vec<RSpecEvent>> {
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

        let mut results: Vec<RSpecEvent> = Vec::new();
        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines {
                match line {
                    Ok(line) => match serde_json::from_str::<RSpecEvent>(&line) {
                        Ok(event) => {
                            results.push(event);
                        }
                        Err(e) => {
                            return Err(anyhow::Error::from(e));
                        }
                    },
                    _ => {}
                }
            }
        }

        cmd.wait()?;

        Ok(results)
    }
}
