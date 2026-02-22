use std::time::Duration;

use log::info;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use exn::Result;
use tokio::time::sleep;

pub async fn create_postgres_pool(
    postgres_user: &'static str,
    postgres_password: &'static str,
    db_address: &'static str,
    db_port: u16,
    postgres_name: &'static str,
    max_conn: u32,
) -> Result<Pool<Postgres>, sqlx::Error> {
    let db_url: String = format!(
        "postgres://{}:{}@{}:{}/{}",
        postgres_user, postgres_password, db_address, db_port, postgres_name
    );
    info!("creating a connection with db: {}", postgres_name);
    info!("postgres connection url: {db_url}");

    let pool = loop {
        let pool = PgPoolOptions::new()
            .max_connections(max_conn)
            .connect(&db_url)
            .await;
        match pool {
            Ok(e) => {
                break e;
            }
            Err(err) => {
                eprintln!("failed to connect to postres: {err}, retrying in a few secs");
                sleep(Duration::from_secs(30)).await;
                continue;
            }
        };
    };

    Ok(pool)
}
