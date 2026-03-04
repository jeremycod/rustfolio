use sqlx::PgPool;
use uuid::Uuid;
use crate::models::alert::User;

pub async fn create_user_with_password(
    pool: &PgPool,
    email: &str,
    name: Option<&str>,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, name, password_hash)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn get_user(pool: &PgPool, user_id: Uuid) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Set (or update) the password hash on an existing user — used to claim a default account.
pub async fn set_user_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
    name: Option<&str>,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET password_hash = $1,
            name = COALESCE($2, name),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(password_hash)
    .bind(name)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Count real users (non-default, with a password hash set)
pub async fn count_non_default_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM users
        WHERE id != '00000000-0000-0000-0000-000000000001'
        AND password_hash IS NOT NULL
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(count.0)
}

/// Migrate all data owned by the default user to a new user.
/// Called when the first real user registers.
pub async fn migrate_default_user_data(
    pool: &PgPool,
    new_user_id: Uuid,
) -> Result<(), sqlx::Error> {
    let default_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE portfolios SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE financial_surveys SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE alert_rules SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE watchlists SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE notifications SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE alert_history SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE notification_preferences SET user_id = $1 WHERE user_id = $2")
        .bind(new_user_id)
        .bind(default_uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
