use std::net::TcpListener;
use prod_craft::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?; 
    run(listener)?.await
}
