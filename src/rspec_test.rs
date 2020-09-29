mod ruby;
use ruby::{RSpec, RSpecConfiguration, RSpecEvent};
use std::sync::mpsc::channel;
use std::thread;

fn main() -> anyhow::Result<()> {
    let mut config = RSpecConfiguration::default();
    config.use_bundler = true;

    let (tx, rx) = channel::<anyhow::Result<RSpecEvent>>();

    let jh = thread::spawn(move || loop {
        let event = rx.recv().unwrap().unwrap();

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
                break;
            }
        }
    });

    let rspec = RSpec::new(config);
    let locations = vec!["test/example_specs.rb"];
    rspec.run(locations, tx.clone())?;
    jh.join().unwrap();

    Ok(())
}
