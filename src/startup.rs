use crate::routes::{confirm, health_check, subscribe, publish_newsletter};
use actix_web::{ web, App, HttpServer };
use actix_web::web::Data;
use actix_web::dev::Server;
use std::net::TcpListener;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::email_client::EmailClient;
use crate::configuration::Settings;
use sqlx::postgres::PgPoolOptions;
use crate::configuration::DatabaseSettings;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    //  converted the `build` function into a constructor for `Application`.
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let email_client = configuration.email_client.client();
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?; 
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener, 
            connection_pool, 
            email_client,
            configuration.application.base_url,
        )?;
        
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
    
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub String);

pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient, base_url: String,) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/newsletters", web::post().to(publish_newsletter))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}