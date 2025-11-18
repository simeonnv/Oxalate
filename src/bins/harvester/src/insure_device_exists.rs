use sqlx::{Pool, Postgres};

pub async fn insure_device_exists(
    machine_id: &str,
    db_pool: &Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    let device_exists = sqlx::query_scalar!(
        "
         SELECT EXISTS (
            SELECT 1 
            FROM Devices 
            WHERE machine_id = $1
        ) AS row_exists;    
    ",
        machine_id
    )
    .fetch_one(db_pool)
    .await?
    .unwrap_or(false);

    if device_exists {
        return Ok(());
    }

    sqlx::query!(
        "
            INSERT INTO Devices
                (machine_id)
            VALUES
                ($1)
            ;
        ",
        machine_id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
