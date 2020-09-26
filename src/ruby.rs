use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;

pub struct RSpec {}

impl RSpec {
    pub fn run() {
        let mut cmd = Command::new("/home/jeremyw/.asdf/shims/ruby")
            .arg("test.rb")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed");

        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines {
                println!("Read: {:?}", line);
            }
        }

        cmd.wait().unwrap();
    }
}
