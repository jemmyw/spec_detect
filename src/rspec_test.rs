mod ruby;
use anyhow::Result;
use ruby::rspec::{RSpec, RSpecConfiguration, RSpecEvent};
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = RSpecConfiguration::default();
    config.use_bundler = true;

    let (tx, mut rx) = channel::<RSpecEvent>(1);

    let jh = tokio::spawn(async move {
        loop {
            let event_result = rx.recv().await;

            if event_result.is_none() {
                println!("oh no no result");
                break;
            }

            let event = event_result.unwrap();

            match event {
                RSpecEvent::Start { count: _ } => println!("Specs started"),
                RSpecEvent::ExampleStarted { description, .. } => {
                    println!("Example started: {:?}", description);
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
        }
    });

    let rspec = RSpec::new(config);
    let locations = vec!["test/example_specs.rb"];

    let run = rspec.run(locations, tx)?;
    run.wait().await.unwrap();
    jh.await.unwrap();

    Ok(())
}
