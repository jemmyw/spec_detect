mod ruby;
use ruby::{RSpec, RSpecConfiguration, RSpecEvent};
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
            RSpecEvent::Start { count } => println!("Specs started"),
            RSpecEvent::ExampleStarted {
                id,
                location,
                description,
            } => {
                println!("Example started");
            }
            RSpecEvent::ExamplePassed {
                id,
                load_time,
                location,
                description,
                run_time,
            } => {
                println!("Example passed");
            }
            RSpecEvent::ExampleFailed {
                id,
                load_time,
                location,
                description,
                run_time,
                exception,
            } => {
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

    let run = rspec.run(locations, tx.clone())?;
    run.wait()?;

    jh.join().unwrap();

    Ok(())
}
