use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, sea_query::Expr,
};

use crate::{
    domain::{
        DomainResult, Error,
        model::{
            permission::{Permission, PermissionCreate},
            user::{User, UserCreate, UserCursor, UserPage, UserQuery, UserSortField, UserUpdate},
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
        query: &UserQuery,
    ) -> DomainResult<UserPage> {
        use crate::domain::model::product::SortDirection;

        let mut select = UserEntity::find().filter(UserColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────────
        let mut condition = Condition::all();

        if let Some(username) = &query.filter.username {
            condition = condition.add(UserColumn::Username.eq(username));
        }

        if let Some(name) = &query.filter.name {
            condition = condition.add(UserColumn::Name.contains(name));
        }

        if let Some(email) = &query.filter.email {
            condition = condition.add(UserColumn::Email.eq(email));
        }

        select = select.filter(condition);

        // ── Map sort direction ────────────────────────────────────────────────
        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ──────────────────────────────────────────────────
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_field {
                // For Id sort, id IS the sort column — no tiebreaker needed
                UserSortField::Id => match query.sort_direction {
                    SortDirection::Asc => {
                        Condition::all().add(Expr::col(UserColumn::Id).gt(cursor.id))
                    }
                    SortDirection::Desc => {
                        Condition::all().add(Expr::col(UserColumn::Id).lt(cursor.id))
                    }
                },
                // For string/date fields: (field > val) OR (field = val AND id > cursor_id)
                UserSortField::UpdatedAt | UserSortField::Name => {
                    let sort_col = match query.sort_field {
                        UserSortField::UpdatedAt => UserColumn::UpdatedAt,
                        UserSortField::Name => UserColumn::Name,
                        UserSortField::Id => unreachable!(),
                    };
                    match query.sort_direction {
                        SortDirection::Asc => Condition::any()
                            .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(UserColumn::Id).gt(cursor.id)),
                            ),
                        SortDirection::Desc => Condition::any()
                            .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                            .add(
                                Condition::all()
                                    .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                                    .add(Expr::col(UserColumn::Id).lt(cursor.id)),
                            ),
                    }
                }
            };
            select = select.filter(cond);
        }

        // ── Ordering: (sort_field, id) ────────────────────────────────────────
        select = match query.sort_field {
            UserSortField::Id => select.order_by(UserColumn::Id, order),
            UserSortField::UpdatedAt => select
                .order_by(UserColumn::UpdatedAt, order.clone())
                .order_by(UserColumn::Id, order),
            UserSortField::Name => select
                .order_by(UserColumn::Name, order.clone())
                .order_by(UserColumn::Id, order),
        };

        // Fetch limit + 1 to detect whether there is a next page
        let fetch_limit = query.limit + 1;
        let rows = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next_cursor from the last item ──────────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    UserSortField::Id => last.id.to_string(),
                    UserSortField::UpdatedAt => last.updated_at.clone(),
                    UserSortField::Name => last.name.clone(),
                };
                UserCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        let items: Vec<User> = models.into_iter().map(|m| m.to_domain()).collect();

        Ok(UserPage { items, next_cursor })
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
        permissions: &[PermissionCreate],
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
