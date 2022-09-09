use termetheus::config::Config;
use termetheus::prometheus::Prometheus;
use termetheus::ui::run;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args)?;
    let prometheus = Prometheus::new(config);

    run(prometheus).await?;

    Ok(())
}
