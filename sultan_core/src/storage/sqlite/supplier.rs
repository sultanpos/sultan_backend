use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter,
    QuerySelect, Set,
};

use super::entity::{SupplierActiveModel, SupplierColumn, SupplierEntity};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::{
            pagination::PaginationOptions,
            supplier::{Supplier, SupplierCreate, SupplierFilter, SupplierUpdate},
        },
    },
    storage::{RepoCtx, SupplierRepository},
};

/// SQLite implementation of [`SupplierRepository`] using SeaORM.
///
/// This repository handles all supplier-related database operations
/// including CRUD operations, soft-delete, and filtered queries.
#[derive(Clone, Default)]
pub struct SqliteSupplierRepository {}

impl SqliteSupplierRepository {
    pub fn new() -> Self {
        SqliteSupplierRepository {}
    }
}

#[async_trait]
impl SupplierRepository for SqliteSupplierRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        supplier: &SupplierCreate,
    ) -> DomainResult<()> {
        let metadata_json = supplier
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let supplier_model = SupplierActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            deleted_at: Set(None),
            is_deleted: Set(false),
            name: Set(supplier.name.clone()),
            code: Set(supplier.code.clone()),
            email: Set(supplier.email.clone()),
            address: Set(supplier.address.clone()),
            phone: Set(supplier.phone.clone()),
            npwp: Set(supplier.npwp.clone()),
            npwp_name: Set(supplier.npwp_name.clone()),
            metadata: Set(metadata_json),
        };

        supplier_model.insert(&ctx.db).await?;
        Ok(())
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        supplier: &SupplierUpdate,
    ) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let mut update_query = SupplierEntity::update_many()
            .filter(SupplierColumn::Id.eq(id))
            .filter(SupplierColumn::IsDeleted.eq(false));

        // Update fields if provided
        if let Some(name) = &supplier.name {
            update_query = update_query.col_expr(SupplierColumn::Name, Expr::value(name.clone()));
        }

        if supplier.code.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Code,
                Expr::value(supplier.code.to_bind_value()),
            );
        }

        if supplier.email.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Email,
                Expr::value(supplier.email.to_bind_value()),
            );
        }

        if supplier.address.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Address,
                Expr::value(supplier.address.to_bind_value()),
            );
        }

        if supplier.phone.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Phone,
                Expr::value(supplier.phone.to_bind_value()),
            );
        }

        if supplier.npwp.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::Npwp,
                Expr::value(supplier.npwp.to_bind_value()),
            );
        }

        if supplier.npwp_name.should_update() {
            update_query = update_query.col_expr(
                SupplierColumn::NpwpName,
                Expr::value(supplier.npwp_name.to_bind_value()),
            );
        }

        if supplier.metadata.should_update() {
            let metadata_json = supplier
                .metadata
                .as_value()
                .map(|m| serde_json::to_string(m).unwrap_or_default());
            update_query =
                update_query.col_expr(SupplierColumn::Metadata, Expr::value(metadata_json));
        }

        // Always update the updated_at timestamp
        update_query = update_query.col_expr(
            SupplierColumn::UpdatedAt,
            Expr::value(
                chrono::Utc::now()
                    .format("%Y-%m-%dT%H:%M:%S%.fZ")
                    .to_string(),
            ),
        );

        // Execute the update
        let result = update_query.exec(&ctx.db).await?;

        // Check if any rows were affected
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Supplier with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        use sea_orm::sea_query::Expr;

        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.fZ")
            .to_string();

        let result = SupplierEntity::update_many()
            .filter(SupplierColumn::Id.eq(id))
            .filter(SupplierColumn::IsDeleted.eq(false))
            .col_expr(SupplierColumn::IsDeleted, Expr::value(true))
            .col_expr(SupplierColumn::DeletedAt, Expr::value(Some(now.clone())))
            .col_expr(SupplierColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Supplier with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        filter: &SupplierFilter,
        pagination: &PaginationOptions,
    ) -> DomainResult<Vec<Supplier>> {
        let mut query = SupplierEntity::find();

        // Build filter conditions
        let mut condition = Condition::all();

        if let Some(name) = &filter.name {
            condition = condition.add(SupplierColumn::Name.contains(name));
        }

        if let Some(code) = &filter.code {
            condition = condition.add(SupplierColumn::Code.contains(code));
        }

        if let Some(email) = &filter.email {
            condition = condition.add(SupplierColumn::Email.contains(email));
        }

        if let Some(phone) = &filter.phone {
            condition = condition.add(SupplierColumn::Phone.contains(phone));
        }

        if let Some(npwp) = &filter.npwp {
            condition = condition.add(SupplierColumn::Npwp.contains(npwp));
        }

        query = query.filter(condition);

        // Apply pagination
        let suppliers = query
            .filter(SupplierColumn::IsDeleted.eq(false))
            .limit(pagination.limit() as u64)
            .offset(pagination.offset() as u64)
            .all(&ctx.db)
            .await?;

        Ok(suppliers.into_iter().map(|s| s.to_domain()).collect())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<Supplier>> {
        let supplier = SupplierEntity::find_by_id(id)
            .filter(SupplierColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        Ok(supplier.map(|s| s.to_domain()))
    }
}
