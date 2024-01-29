use std::net::TcpListener;
use prod_craft::startup::run;
use sqlx::postgres::PgPool;
use prod_craft::configuration::get_configuration;
use prod_craft::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("prod_craft".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?; 
    run(listener, connection_pool)?.await?;
    Ok(())
}