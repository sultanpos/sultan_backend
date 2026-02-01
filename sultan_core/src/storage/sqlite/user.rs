use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::{
            pagination::PaginationOptions,
            permission::Permission,
            user::{User, UserCreate, UserFilter, UserUpdate},
        },
    },
    storage::{
        RepoCtx,
        sqlite::entity::{
            PermissionActiveModel, PermissionColumn, PermissionEntity, UserActiveModel, UserColumn,
            UserEntity,
        },
        user_repo::UserRepository,
    },
};

// ============================================================================
// SQLite User Repository
// ============================================================================

/// SQLite implementation of [`UserRepository`] using SeaORM.
///
/// This repository uses SeaORM's `ConnectionTrait` which allows it to work
/// with both direct database connections and transactions seamlessly.
///
/// # Example
///
/// ```rust,ignore
/// // Using with direct connection
/// let repo = SqliteUserRepository::new();
/// let ctx = RepoCtx { ctx: Context::new(), db: &db_connection };
/// repo.create(&ctx, id, &user).await?;
///
/// // Using within a transaction
/// let txn = db.begin().await?;
/// let ctx = RepoCtx { ctx: Context::new(), db: &txn };
/// repo.create(&ctx, id, &user).await?;
/// txn.commit().await?;
/// ```
#[derive(Clone, Default)]
pub struct SqliteUserRepository {}

impl SqliteUserRepository {
    pub fn new() -> Self {
        SqliteUserRepository {}
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        user: &UserCreate,
    ) -> DomainResult<()> {
        let user_model = UserActiveModel {
            id: Set(id),
            username: Set(user.username.clone()),
            password: Set(user.password.clone()),
            name: Set(user.name.clone()),
            email: Set(user.email.clone()),
            photo: Set(user.photo.clone()),
            pin: Set(user.pin.clone()),
            address: Set(user.address.clone()),
            phone: Set(user.phone.clone()),
            ..Default::default()
        };

        user_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn get_by_username(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        username: &str,
    ) -> DomainResult<Option<User>> {
        let user = UserEntity::find()
            .filter(UserColumn::Username.eq(username))
            .filter(UserColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(user.map(|u| u.to_domain()))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        user: &UserUpdate,
    ) -> DomainResult<()> {
        use sea_orm::{UpdateMany, sea_query::Expr};

        let mut update_query: UpdateMany<UserEntity> = UserEntity::update_many()
            .filter(UserColumn::Id.eq(id))
            .filter(UserColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(username) = &user.username {
            update_query =
                update_query.col_expr(UserColumn::Username, Expr::value(username.clone()));
        }

        if let Some(name) = &user.name {
            update_query = update_query.col_expr(UserColumn::Name, Expr::value(name.clone()));
        }

        if user.email.should_update() {
            update_query =
                update_query.col_expr(UserColumn::Email, Expr::value(user.email.to_bind_value()));
        }

        if user.photo.should_update() {
            update_query =
                update_query.col_expr(UserColumn::Photo, Expr::value(user.photo.to_bind_value()));
        }

        if user.pin.should_update() {
            update_query =
                update_query.col_expr(UserColumn::Pin, Expr::value(user.pin.to_bind_value()));
        }

        if user.address.should_update() {
            update_query = update_query.col_expr(
                UserColumn::Address,
                Expr::value(user.address.to_bind_value()),
            );
        }

        if user.phone.should_update() {
            update_query =
                update_query.col_expr(UserColumn::Phone, Expr::value(user.phone.to_bind_value()));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            UserColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        let result = update_query.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("User with id {} not found", id)));
        }

        Ok(())
    }

    async fn update_password(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        password_hash: &str,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let result = UserEntity::update_many()
            .filter(UserColumn::Id.eq(id))
            .col_expr(UserColumn::Password, Expr::value(password_hash))
            .col_expr(
                UserColumn::UpdatedAt,
                Expr::value(
                    chrono::Utc::now()
                        .format("%Y-%m-%dT%H:%M:%S%.fZ")
                        .to_string(),
                ),
            )
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("User with id {} not found", id)));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let result = UserEntity::update_many()
            .filter(UserColumn::Id.eq(id))
            .filter(UserColumn::IsDeleted.eq(false))
            .col_expr(UserColumn::IsDeleted, Expr::value(true))
            .col_expr(UserColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(UserColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!("User with id {} not found", id)));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        filter: &UserFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<User>> {
        let mut query = UserEntity::find().filter(UserColumn::IsDeleted.eq(false));

        // Apply filters
        if let Some(username) = &filter.username {
            query = query.filter(UserColumn::Username.eq(username));
        }

        if let Some(name) = &filter.name {
            query = query.filter(UserColumn::Name.contains(name));
        }

        if let Some(email) = &filter.email {
            query = query.filter(UserColumn::Email.eq(email));
        }

        // Apply pagination
        let limit = pagination.limit();
        let offset = pagination.offset();

        let users = query
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&ctx.db)
            .await?;

        Ok(users.into_iter().map(|u| u.to_domain()).collect())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<User>> {
        let user = UserEntity::find_by_id(id)
            .filter(UserColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(user.map(|u| u.to_domain()))
    }

    async fn delete_permission_by_user_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        user_id: i64,
    ) -> DomainResult<()> {
        PermissionEntity::delete_many()
            .filter(PermissionColumn::UserId.eq(user_id))
            .exec(&ctx.db)
            .await?;
        Ok(())
    }

    async fn save_permissions(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        user_id: i64,
        permissions: &[Permission],
    ) -> DomainResult<()> {
        for perm in permissions {
            let permission_model = PermissionActiveModel {
                user_id: Set(user_id),
                branch_id: Set(perm.branch_id),
                resource: Set(perm.resource),
                action: Set(perm.action),
                ..Default::default()
            };

            permission_model.insert(&ctx.db).await?;
        }
        Ok(())
    }

    async fn get_permissions(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        user_id: i64,
    ) -> DomainResult<Vec<Permission>> {
        let permissions = PermissionEntity::find()
            .filter(PermissionColumn::UserId.eq(user_id))
            .all(&ctx.db)
            .await?;

        Ok(permissions.into_iter().map(|p| p.to_domain()).collect())
    }
}
