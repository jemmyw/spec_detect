mod ruby;
use ruby::rspec::{RSpec, RSpecConfiguration, RSpecEvent};
use std::sync::mpsc::channel;
use std::thread;

fn main() -> anyhow::Result<()> {
    let mut config = RSpecConfiguration::default();
    config.use_bundler = true;

    let (tx, rx) = channel::<RSpecEvent>();

    let jh = thread::spawn(move || loop {
        let event_result = rx.recv();

        if event_result.is_err() {
            println!("oh no an error on rx");
            break;
        }

        let event = event_result.unwrap();

        match event {
            RSpecEvent::Start { count: _ } => println!("Specs started"),
            RSpecEvent::ExampleStarted { .. } => {
                println!("Example started");
            }
            RSpecEvent::ExamplePassed { .. } => {
                println!("Example passed");
            }
            RSpecEvent::ExampleFailed { .. } => {
                println!("Example failed");
            }
            RSpecEvent::Stop {} => {
                println!("Done");
            }
            RSpecEvent::Exit => {
                println!("Exit");
                break;
            }
            RSpecEvent::Error { msg } => {
                println!("RSpec error {}", msg);
            }
        }
    });

    let rspec = RSpec::new(config);
    let locations = vec!["test/example_specs.rb"];

    let run = rspec.run(locations, tx)?;
    run.wait()?;

    jh.join().unwrap();

    Ok(())
}
