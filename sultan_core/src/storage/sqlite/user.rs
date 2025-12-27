use async_trait::async_trait;

use serde::Serialize;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::{
    domain::{
        Context, DomainResult, Error,
        model::{
            pagination::PaginationOptions,
            permission::Permission,
            user::{User, UserCreate, UserFilter, UserUpdate},
        },
    },
    storage::user_repo::UserRepository,
};

// ============================================================================
// SQLite User Repository
// ============================================================================

const USER_COLUMNS: &str = "id, username, email, password, name, created_at, updated_at, deleted_at, is_deleted, photo, pin, address, phone";

#[derive(Clone)]
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Check if a query affected rows, return error if not
    fn check_rows_affected(
        rows: u64,
        entity: &str,
        id: impl std::fmt::Display,
    ) -> DomainResult<()> {
        if rows == 0 {
            return Err(Error::NotFound(format!(
                "{} with id {} not found",
                entity, id
            )));
        }
        Ok(())
    }
}

// Database model for User - SQLite
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct UserDbSqlite {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub is_deleted: bool,
    pub photo: Option<String>,
    pub pin: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
}

impl From<UserDbSqlite> for User {
    fn from(user_db: UserDbSqlite) -> Self {
        User {
            id: user_db.id,
            username: user_db.username,
            email: user_db.email,
            password: user_db.password,
            name: user_db.name,
            created_at: super::parse_sqlite_date(&user_db.created_at),
            updated_at: super::parse_sqlite_date(&user_db.updated_at),
            deleted_at: user_db.deleted_at.map(|d| super::parse_sqlite_date(&d)),
            is_deleted: user_db.is_deleted,
            photo: user_db.photo,
            pin: user_db.pin,
            address: user_db.address,
            phone: user_db.phone,
            permissions: None,
        }
    }
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct PermissionDbSqlite {
    pub id: i64,
    pub user_id: i64,
    pub branch_id: Option<i64>,
    pub permission: i32,
    pub action: i32,
}

impl From<PermissionDbSqlite> for Permission {
    fn from(permission_db: PermissionDbSqlite) -> Self {
        Permission {
            user_id: permission_db.user_id,
            branch_id: permission_db.branch_id,
            permission: permission_db.permission,
            action: permission_db.action,
        }
    }
}

// Implement the UserRepository trait for SQLite
#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn create_user(&self, _: &Context, id: i64, user: &UserCreate) -> DomainResult<()> {
        let query = sqlx::query("INSERT INTO users (id, username, name, email, password, photo, pin, address, phone) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(id)
            .bind(&user.username)
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password)
            .bind(&user.photo)
            .bind(&user.pin)
            .bind(&user.address)
            .bind(&user.phone)
            .execute(&self.pool);

        query.await?;
        Ok(())
    }

    async fn get_user_by_username(
        &self,
        _: &Context,
        username: &str,
    ) -> DomainResult<Option<User>> {
        let sql = format!(
            "SELECT {} FROM users WHERE username = ? AND is_deleted = 0",
            USER_COLUMNS
        );
        let query = sqlx::query_as::<_, UserDbSqlite>(&sql)
            .bind(username)
            .fetch_optional(&self.pool);

        Ok(query.await?.map(User::from))
    }

    async fn update_user(&self, _: &Context, id: i64, user: &UserUpdate) -> DomainResult<()> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE users SET ");
        let mut separated = builder.separated(", ");

        if let Some(username) = &user.username {
            separated
                .push("username = ")
                .push_bind_unseparated(username);
        }
        if let Some(name) = &user.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if user.email.should_update() {
            separated
                .push("email = ")
                .push_bind_unseparated(user.email.to_bind_value());
        }
        if user.photo.should_update() {
            separated
                .push("photo = ")
                .push_bind_unseparated(user.photo.to_bind_value());
        }
        if user.pin.should_update() {
            separated
                .push("pin = ")
                .push_bind_unseparated(user.pin.to_bind_value());
        }
        if user.address.should_update() {
            separated
                .push("address = ")
                .push_bind_unseparated(user.address.to_bind_value());
        }
        if user.phone.should_update() {
            separated
                .push("phone = ")
                .push_bind_unseparated(user.phone.to_bind_value());
        }
        separated.push("updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')");
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" AND is_deleted = 0");

        let query = builder.build();
        let result = query.execute(&self.pool).await?;
        Self::check_rows_affected(result.rows_affected(), "User", id)
    }

    async fn update_password(&self, _: &Context, id: i64, password_hash: &str) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            UPDATE users SET
                password = ?,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?
            "#,
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.pool);

        let result = query.await?;
        Self::check_rows_affected(result.rows_affected(), "User", id)
    }

    async fn delete_user(&self, _: &Context, user_id: i64) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            UPDATE users SET
                is_deleted = 1,
                deleted_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ? AND is_deleted = 0
            "#,
        )
        .bind(user_id)
        .execute(&self.pool);

        let result = query.await?;
        Self::check_rows_affected(result.rows_affected(), "User", user_id)
    }

    async fn get_all(
        &self,
        _: &Context,
        filter: UserFilter,
        pagination: PaginationOptions,
    ) -> DomainResult<Vec<User>> {
        let limit = pagination.limit();
        let offset = pagination.offset();

        let mut sql = format!("SELECT {} FROM users WHERE is_deleted = 0", USER_COLUMNS);
        let mut bindings: Vec<String> = Vec::new();

        if let Some(ref username) = filter.username {
            sql.push_str(" AND username LIKE ?");
            bindings.push(format!("{}%", username));
        }

        if let Some(ref name) = filter.name {
            sql.push_str(" AND name LIKE ?");
            bindings.push(format!("%{}%", name));
        }

        if let Some(ref email) = filter.email {
            sql.push_str(" AND email = ?");
            bindings.push(email.to_string());
        }

        sql.push_str(" LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, UserDbSqlite>(&sql);

        for binding in &bindings {
            query = query.bind(binding);
        }

        query = query.bind(limit).bind(offset);

        let query = query.fetch_all(&self.pool);

        let users = query.await?;
        Ok(users.into_iter().map(User::from).collect())
    }

    async fn get_by_id(&self, _: &Context, user_id: i64) -> DomainResult<Option<User>> {
        let sql = format!(
            "SELECT {} FROM users WHERE id = ? AND is_deleted = 0",
            USER_COLUMNS
        );
        let query = sqlx::query_as::<_, UserDbSqlite>(&sql)
            .bind(user_id)
            .fetch_optional(&self.pool);

        Ok(query.await?.map(User::from))
    }

    async fn save_user_permission(
        &self,
        _: &Context,
        user_id: i64,
        branch_id: Option<i64>,
        permission: i32,
        action: i32,
    ) -> DomainResult<()> {
        // First, try to delete existing permission
        // Use proper NULL handling for comparison
        let delete_query = sqlx::query(
            r#"
            DELETE FROM permissions
            WHERE user_id = ? AND permission = ? AND (
                (branch_id IS NULL AND ? IS NULL) OR
                (branch_id = ? AND ? IS NOT NULL)
            )
            "#,
        )
        .bind(user_id)
        .bind(permission)
        .bind(branch_id)
        .bind(branch_id)
        .bind(branch_id)
        .execute(&self.pool);

        delete_query.await.ok();

        let insert_query = sqlx::query(
            r#"
            INSERT INTO permissions (user_id, branch_id, permission, action)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(branch_id)
        .bind(permission)
        .bind(action)
        .execute(&self.pool);

        insert_query.await?;
        Ok(())
    }

    async fn delete_user_permission(
        &self,
        _: &Context,
        user_id: i64,
        branch_id: Option<i64>,
        permission: i32,
    ) -> DomainResult<()> {
        let query = sqlx::query(
            r#"
            DELETE FROM permissions
            WHERE user_id = ? AND permission = ? AND (
                (branch_id IS NULL AND ? IS NULL) OR
                (branch_id = ? AND ? IS NOT NULL)
            )
            "#,
        )
        .bind(user_id)
        .bind(permission)
        .bind(branch_id)
        .bind(branch_id)
        .bind(branch_id)
        .execute(&self.pool);

        let result = query.await?;
        Self::check_rows_affected(
            result.rows_affected(),
            "Permission",
            format!(
                "user_id: {}, permission: {}, branch_id: {:?}",
                user_id, permission, branch_id
            ),
        )
    }

    async fn get_user_permission(
        &self,
        _: &Context,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>> {
        let sql = "SELECT * FROM permissions WHERE user_id = ?";
        let query = sqlx::query_as::<_, PermissionDbSqlite>(sql)
            .bind(user_id)
            .fetch_all(&self.pool);

        let permissions_db = query.await?;
        Ok(permissions_db.into_iter().map(Permission::from).collect())
    }
}
