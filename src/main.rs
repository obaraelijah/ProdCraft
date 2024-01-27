use std::net::TcpListener;
use prod_craft::startup::run;
use prod_craft::configuration::get_configuration;
use sqlx::PgPool;
use env_logger::Env;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    let configuration = get_configuration().expect("Failed toread configuration");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?; 
    run(listener, connection)?.await
}
