mod ruby;
use ruby::{RSpec, RSpecConfiguration};

fn main() -> anyhow::Result<()> {
    let mut config = RSpecConfiguration::default();
    config.use_bundler = true;

    let rspec = RSpec::new(config);
    let locations = vec!["test/example_specs.rb"];
    let output = rspec.run(locations)?;

    dbg!(output);

    Ok(())
}
