use crate::routes::{confirm, health_check, subscribe, publish_newsletter};
use crate::routes::{home, login_form, login, admin_dashboard, change_password, change_password_form};
use crate::email_client::EmailClient;
use crate::configuration::Settings;
use crate::configuration::DatabaseSettings;
use actix_web::{ web, App, HttpServer };
use actix_web::web::Data;
use actix_web::dev::Server;
use std::net::TcpListener;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use sqlx::postgres::PgPoolOptions;
use secrecy::{Secret, ExposeSecret};
use actix_web_flash_messages::FlashMessagesFramework;
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web::cookie::Key;
use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let email_client = configuration.email_client.client();
        let address = format!("{}:{}", configuration.application.host, configuration.application.port);
        let listener = TcpListener::bind(&address)?; 
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener, 
            connection_pool, 
            email_client,
            configuration.application.base_url,
            HmacSecret(configuration.application.hmac_secret.clone()),
            configuration.redis_uri,
        ).await?;
        
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

async fn run(
    listener: TcpListener, 
    db_pool: PgPool, 
    email_client: EmailClient, 
    base_url: String, 
    hmac_secret: HmacSecret,
    redis_uri: Secret<String>,
) -> Result<Server, anyhow::Error> {
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let secret_key = Key::from(hmac_secret.0.expose_secret().as_bytes());
    let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();
    let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;
    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .route("/admin/dashboard", web::get().to(admin_dashboard))
            .route("/admin/password", web::get().to(change_password_form))
            .route("/admin/password", web::post().to(change_password))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            .app_data(Data::new(hmac_secret.0.clone()))
            .wrap(message_framework.clone())
            .wrap(SessionMiddleware::new(redis_store.clone(), secret_key.clone()))
            .wrap(TracingLogger::default())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

#[derive(Clone, Debug)]
pub struct HmacSecret(pub Secret<String>);
