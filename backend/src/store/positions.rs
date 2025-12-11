use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_position(
    pool: &PgPool,
    portfolio_id: Uuid,
    ticker: String,
    shares: f32,
    avg_price: f32,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query!(
        "
        INSERT INTO positions (id, portfolio_id, ticker, shares, avg_buy_price)
        VALUES ($1, $2, $3, $4::REAL, $5::REAL)
        ",
        id,
        portfolio_id,
        ticker,
        shares,
        avg_price
    )
    .execute(pool)
    .await?;

    Ok(id)
}