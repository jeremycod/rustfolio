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

/// Update a user's profile (email and/or name).
pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    name: Option<&str>,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET email = $1, name = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(email)
    .bind(name)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Update a user's password hash.
pub async fn update_user_password(
    pool: &PgPool,
    user_id: Uuid,
    new_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(new_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Create a password reset token for a user (1-hour expiry). Replaces any existing token.
pub async fn create_password_reset_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO password_reset_tokens (token, user_id, expires_at) \
         VALUES ($1, $2, NOW() + INTERVAL '1 hour')",
    )
    .bind(token)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Consume a password reset token (delete it) and return the user_id if valid and not expired.
pub async fn consume_password_reset_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "DELETE FROM password_reset_tokens \
         WHERE token = $1 AND expires_at > NOW() \
         RETURNING user_id",
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(user_id,)| user_id))
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
