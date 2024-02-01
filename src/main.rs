use prod_craft::configuration::get_configuration;
use prod_craft::telemetry::{get_subscriber, init_subscriber};
use prod_craft::startup::build;
use prod_craft::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("prod_craft".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let server = build(configuration).await?;
    server.await?;
    let application = Application::build(configuration).await?;
    application.run_untill_stopped().await?;
    Ok(())
}