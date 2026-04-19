use std::collections::HashMap;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, ExprTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set, Statement, sea_query::Expr,
};

use super::entity::{
    PurchaseOrderActiveModel, PurchaseOrderColumn, PurchaseOrderEntity,
    PurchaseOrderItemActiveModel, PurchaseOrderItemColumn, PurchaseOrderItemEntity,
    PurchasePaymentActiveModel, PurchasePaymentColumn, PurchasePaymentEntity,
};
use crate::{
    domain::{
        DomainResult,
        error::Error,
        model::{
            product::SortDirection,
            purchase_order::{
                PaymentStatus, PurchaseOrder, PurchaseOrderCreate, PurchaseOrderCursor,
                PurchaseOrderItem, PurchaseOrderItemCreate, PurchaseOrderItemUpdate,
                PurchaseOrderPage, PurchaseOrderQuery, PurchaseOrderSortField, PurchaseOrderUpdate,
                PurchasePayment, PurchasePaymentCreate, PurchasePaymentUpdate,
            },
        },
    },
    storage::{PurchaseOrderRepository, RepoCtx},
};

#[derive(Clone, Default)]
pub struct SqlitePurchaseOrderRepository {}

impl SqlitePurchaseOrderRepository {
    pub fn new() -> Self {
        SqlitePurchaseOrderRepository {}
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_str() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.fZ")
        .to_string()
}

async fn fetch_items_for_orders<C: ConnectionTrait>(
    db: &C,
    order_ids: &[i64],
) -> DomainResult<Vec<super::entity::purchase_order_item::Model>> {
    if order_ids.is_empty() {
        return Ok(vec![]);
    }
    let items = PurchaseOrderItemEntity::find()
        .filter(PurchaseOrderItemColumn::PurchaseOrderId.is_in(order_ids.to_vec()))
        .all(db)
        .await?;
    Ok(items)
}

async fn fetch_payments_for_orders<C: ConnectionTrait>(
    db: &C,
    order_ids: &[i64],
) -> DomainResult<Vec<super::entity::purchase_payment::Model>> {
    if order_ids.is_empty() {
        return Ok(vec![]);
    }
    let payments = PurchasePaymentEntity::find()
        .filter(PurchasePaymentColumn::PurchaseOrderId.is_in(order_ids.to_vec()))
        .all(db)
        .await?;
    Ok(payments)
}

fn assemble_orders(
    order_models: Vec<super::entity::purchase_order::Model>,
    item_models: Vec<super::entity::purchase_order_item::Model>,
    payment_models: Vec<super::entity::purchase_payment::Model>,
) -> Vec<PurchaseOrder> {
    // Pre-group by purchase_order_id so assembly is O(#orders + #items + #payments)
    // rather than O(#orders × (#items + #payments)).
    let mut items_by_order: HashMap<i64, Vec<PurchaseOrderItem>> =
        HashMap::with_capacity(order_models.len());
    for item in item_models {
        items_by_order
            .entry(item.purchase_order_id)
            .or_default()
            .push(item.to_domain());
    }

    let mut payments_by_order: HashMap<i64, Vec<PurchasePayment>> =
        HashMap::with_capacity(order_models.len());
    for payment in payment_models {
        payments_by_order
            .entry(payment.purchase_order_id)
            .or_default()
            .push(payment.to_domain());
    }

    order_models
        .into_iter()
        .map(|o| {
            let items = items_by_order.remove(&o.id).unwrap_or_default();
            let payments = payments_by_order.remove(&o.id).unwrap_or_default();
            o.to_domain(items, payments)
        })
        .collect()
}

// ── Implementation ────────────────────────────────────────────────────────────

#[async_trait]
impl PurchaseOrderRepository for SqlitePurchaseOrderRepository {
    async fn create(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
        data: &PurchaseOrderCreate,
    ) -> DomainResult<()> {
        let now = now_str();

        let order = PurchaseOrderActiveModel {
            id: Set(id),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
            deleted_at: Set(None),
            is_deleted: Set(false),
            branch_id: Set(data.branch_id),
            supplier_id: Set(data.supplier_id),
            number: Set(data.number.clone()),
            reference_number: Set(data.reference_number.clone()),
            status: Set(
                crate::domain::model::purchase_order::PurchaseOrderStatus::Draft
                    .as_str()
                    .to_string(),
            ),
            order_date: Set(data.order_date.clone()),
            expected_date: Set(data.expected_date.clone()),
            received_date: Set(None),
            subtotal: Set(0),
            discount_amount: Set(data.discount_amount),
            total_amount: Set(0),
            payment_status: Set(PaymentStatus::Unpaid.as_str().to_string()),
            payment_due_date: Set(data.payment_due_date.clone()),
            paid_amount: Set(0),
            returned_amount: Set(0),
            notes: Set(data.notes.clone()),
            metadata: Set(data
                .metadata
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok())),
        };
        order.insert(&ctx.db).await?;
        Ok(())
    }

    async fn add_payment(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
        payment_id: i64,
        data: &PurchasePaymentCreate,
    ) -> DomainResult<()> {
        let now = now_str();

        // Atomically increment paid_amount and derive payment_status in one
        // statement — eliminates the read-modify-write race under concurrent
        // payments.  The CASE references the *pre-update* paid_amount value,
        // which equals (old_paid + data.amount) when evaluated by SQLite.
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            "UPDATE purchase_orders \
             SET paid_amount    = paid_amount + ?, \
                 payment_status = CASE \
                     WHEN paid_amount + ? >= total_amount THEN 'paid' \
                     WHEN paid_amount + ?  > 0            THEN 'partial' \
                     ELSE 'unpaid' \
                 END, \
                 updated_at = ? \
             WHERE id = ? AND is_deleted = 0",
            [
                data.amount.into(),
                data.amount.into(),
                data.amount.into(),
                now.clone().into(),
                purchase_order_id.into(),
            ],
        );
        let result = ctx.db.execute_raw(stmt).await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!(
                "Purchase order with id {} not found",
                purchase_order_id
            )));
        }

        let payment = PurchasePaymentActiveModel {
            id: Set(payment_id),
            created_at: Set(now),
            purchase_order_id: Set(purchase_order_id),
            amount: Set(data.amount),
            payment_channel_id: Set(data.payment_channel_id),
            paid_at: Set(data.paid_at.clone()),
            reference: Set(data.reference.clone()),
            notes: Set(data.notes.clone()),
        };
        payment.insert(&ctx.db).await?;

        Ok(())
    }

    async fn get_by_id(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        id: i64,
    ) -> DomainResult<Option<PurchaseOrder>> {
        let order = PurchaseOrderEntity::find_by_id(id)
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;

        let Some(order) = order else {
            return Ok(None);
        };

        let item_models = fetch_items_for_orders(&ctx.db, &[id]).await?;
        let payment_models = fetch_payments_for_orders(&ctx.db, &[id]).await?;

        let items = item_models.into_iter().map(|i| i.to_domain()).collect();
        let payments = payment_models.into_iter().map(|p| p.to_domain()).collect();

        Ok(Some(order.to_domain(items, payments)))
    }

    async fn update(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        branch_id: i64,
        id: i64,
        data: &PurchaseOrderUpdate,
    ) -> DomainResult<()> {
        let now = now_str();

        let mut q = PurchaseOrderEntity::update_many()
            .filter(PurchaseOrderColumn::Id.eq(id))
            .filter(PurchaseOrderColumn::BranchId.eq(branch_id))
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now));

        if let Some(supplier_id) = data.supplier_id {
            q = q.col_expr(PurchaseOrderColumn::SupplierId, Expr::value(supplier_id));
        }
        if data.reference_number.should_update() {
            q = q.col_expr(
                PurchaseOrderColumn::ReferenceNumber,
                Expr::value(data.reference_number.to_bind_value()),
            );
        }
        if let Some(status) = &data.status {
            q = q.col_expr(PurchaseOrderColumn::Status, Expr::value(status.as_str()));
        }
        if data.order_date.should_update() {
            q = q.col_expr(
                PurchaseOrderColumn::OrderDate,
                Expr::value(data.order_date.to_bind_value()),
            );
        }
        if data.expected_date.should_update() {
            q = q.col_expr(
                PurchaseOrderColumn::ExpectedDate,
                Expr::value(data.expected_date.to_bind_value()),
            );
        }
        if data.received_date.should_update() {
            q = q.col_expr(
                PurchaseOrderColumn::ReceivedDate,
                Expr::value(data.received_date.to_bind_value()),
            );
        }
        if let Some(subtotal) = data.subtotal {
            q = q.col_expr(PurchaseOrderColumn::Subtotal, Expr::value(subtotal));
        }
        if let Some(discount_amount) = data.discount_amount {
            q = q.col_expr(
                PurchaseOrderColumn::DiscountAmount,
                Expr::value(discount_amount),
            );
        }
        if let Some(total_amount) = data.total_amount {
            q = q.col_expr(PurchaseOrderColumn::TotalAmount, Expr::value(total_amount));
        }
        if let Some(payment_status) = &data.payment_status {
            q = q.col_expr(
                PurchaseOrderColumn::PaymentStatus,
                Expr::value(payment_status.as_str()),
            );
        }
        if data.payment_due_date.should_update() {
            q = q.col_expr(
                PurchaseOrderColumn::PaymentDueDate,
                Expr::value(data.payment_due_date.to_bind_value()),
            );
        }
        if let Some(paid_amount) = data.paid_amount {
            q = q.col_expr(PurchaseOrderColumn::PaidAmount, Expr::value(paid_amount));
        }
        if let Some(returned_amount) = data.returned_amount {
            q = q.col_expr(
                PurchaseOrderColumn::ReturnedAmount,
                Expr::value(returned_amount),
            );
        }
        if data.notes.should_update() {
            q = q.col_expr(
                PurchaseOrderColumn::Notes,
                Expr::value(data.notes.to_bind_value()),
            );
        }
        if data.metadata.should_update() {
            use crate::domain::model::Update;
            let metadata_str: Option<String> = match &data.metadata {
                Update::Set(v) => serde_json::to_string(v).ok(),
                Update::Clear => None,
                Update::Unchanged => unreachable!(),
            };
            q = q.col_expr(PurchaseOrderColumn::Metadata, Expr::value(metadata_str));
        }

        let result = q.exec(&ctx.db).await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Purchase order with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn delete(&self, ctx: &RepoCtx<impl ConnectionTrait>, id: i64) -> DomainResult<()> {
        let now = now_str();

        let result = PurchaseOrderEntity::update_many()
            .filter(PurchaseOrderColumn::Id.eq(id))
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .col_expr(PurchaseOrderColumn::IsDeleted, Expr::value(true))
            .col_expr(
                PurchaseOrderColumn::DeletedAt,
                Expr::value(Some(now.clone())),
            )
            .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Purchase order with id {} not found",
                id
            )));
        }

        Ok(())
    }

    async fn get_all(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        query: &PurchaseOrderQuery,
    ) -> DomainResult<PurchaseOrderPage> {
        let mut select =
            PurchaseOrderEntity::find().filter(PurchaseOrderColumn::IsDeleted.eq(false));

        // ── Filters ──────────────────────────────────────────────────────────
        if let Some(supplier_id) = query.filter.supplier_id {
            select = select.filter(PurchaseOrderColumn::SupplierId.eq(supplier_id));
        }
        if let Some(status) = &query.filter.status {
            select = select.filter(PurchaseOrderColumn::Status.eq(status.as_str()));
        }
        if let Some(number) = &query.filter.number {
            select = select.filter(PurchaseOrderColumn::Number.contains(number.as_str()));
        }
        if let Some(ref_num) = &query.filter.reference_number {
            select = select.filter(PurchaseOrderColumn::ReferenceNumber.contains(ref_num.as_str()));
        }

        // ── Sort column ───────────────────────────────────────────────────────
        let sort_col = match query.sort_field {
            PurchaseOrderSortField::CreatedAt => PurchaseOrderColumn::CreatedAt,
            PurchaseOrderSortField::OrderDate => PurchaseOrderColumn::OrderDate,
            PurchaseOrderSortField::PaymentDueDate => PurchaseOrderColumn::PaymentDueDate,
        };

        // Exclude rows where the sort column is NULL. Nullable columns produce
        // unstable ordering (SQLite places NULLs first in ASC, last in DESC),
        // and an empty-string cursor cannot represent a missing value in
        // subsequent GT/LT comparisons.
        match query.sort_field {
            PurchaseOrderSortField::OrderDate => {
                select = select.filter(PurchaseOrderColumn::OrderDate.is_not_null());
            }
            PurchaseOrderSortField::PaymentDueDate => {
                select = select.filter(PurchaseOrderColumn::PaymentDueDate.is_not_null());
            }
            PurchaseOrderSortField::CreatedAt => {}
        }

        let order = match query.sort_direction {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        };

        // ── Cursor condition ─────────────────────────────────────────────────
        if let Some(cursor) = &query.cursor {
            let cond = match query.sort_direction {
                SortDirection::Asc => Condition::any()
                    .add(Expr::col(sort_col).gt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(PurchaseOrderColumn::Id).gt(cursor.id)),
                    ),
                SortDirection::Desc => Condition::any()
                    .add(Expr::col(sort_col).lt(cursor.field_value.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col(sort_col).eq(cursor.field_value.clone()))
                            .add(Expr::col(PurchaseOrderColumn::Id).lt(cursor.id)),
                    ),
            };
            select = select.filter(cond);
        }

        select = select
            .order_by(sort_col, order.clone())
            .order_by(PurchaseOrderColumn::Id, order);

        let fetch_limit = query.limit + 1;
        let rows = select.limit(fetch_limit).all(&ctx.db).await?;

        let has_next = rows.len() as u64 > query.limit;
        let models: Vec<_> = rows.into_iter().take(query.limit as usize).collect();

        // ── Build next cursor ─────────────────────────────────────────────────
        let next_cursor = if has_next {
            models.last().map(|last| {
                let field_value = match query.sort_field {
                    PurchaseOrderSortField::CreatedAt => last.created_at.clone(),
                    PurchaseOrderSortField::OrderDate => {
                        last.order_date.clone().unwrap_or_default()
                    }
                    PurchaseOrderSortField::PaymentDueDate => {
                        last.payment_due_date.clone().unwrap_or_default()
                    }
                };
                PurchaseOrderCursor {
                    field_value,
                    id: last.id,
                }
            })
        } else {
            None
        };

        // ── Batch-fetch relations ─────────────────────────────────────────────
        let order_ids: Vec<i64> = models.iter().map(|m| m.id).collect();
        let item_models = fetch_items_for_orders(&ctx.db, &order_ids).await?;
        let payment_models = fetch_payments_for_orders(&ctx.db, &order_ids).await?;

        let items = assemble_orders(models, item_models, payment_models);

        Ok(PurchaseOrderPage { items, next_cursor })
    }

    // ── Item management ───────────────────────────────────────────────────────

    async fn add_item(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
        item_id: i64,
        data: &PurchaseOrderItemCreate,
    ) -> DomainResult<()> {
        let now = now_str();

        // Verify order exists and is not deleted
        let order = PurchaseOrderEntity::find_by_id(purchase_order_id)
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Purchase order with id {} not found",
                    purchase_order_id
                ))
            })?;

        let item = PurchaseOrderItemActiveModel {
            id: Set(item_id),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
            purchase_order_id: Set(purchase_order_id),
            product_variant_id: Set(data.product_variant_id),
            product_name: Set(data.product_name.clone()),
            variant_name: Set(data.variant_name.clone()),
            barcode: Set(data.barcode.clone()),
            quantity: Set(data.quantity),
            unit_cost: Set(data.unit_cost),
            discount_amount: Set(data.discount_amount),
            total_cost: Set(data.total_cost()),
            metadata: Set(None),
        };
        item.insert(&ctx.db).await?;

        let new_subtotal = order.subtotal + data.total_cost();
        let new_total = new_subtotal - order.discount_amount;

        PurchaseOrderEntity::update_many()
            .filter(PurchaseOrderColumn::Id.eq(purchase_order_id))
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .col_expr(PurchaseOrderColumn::Subtotal, Expr::value(new_subtotal))
            .col_expr(PurchaseOrderColumn::TotalAmount, Expr::value(new_total))
            .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now))
            .exec(&ctx.db)
            .await?;

        Ok(())
    }

    async fn update_item(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        item_id: i64,
        data: &PurchaseOrderItemUpdate,
    ) -> DomainResult<()> {
        use crate::domain::model::Update;

        let now = now_str();

        // Fetch the existing item to get purchase_order_id and old total_cost
        let existing = PurchaseOrderItemEntity::find_by_id(item_id)
            .one(&ctx.db)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Purchase order item with id {} not found", item_id))
            })?;

        let new_quantity = data.quantity.unwrap_or(existing.quantity);
        let new_unit_cost = data.unit_cost.unwrap_or(existing.unit_cost);
        let new_discount = data.discount_amount.unwrap_or(existing.discount_amount);
        let new_total_cost = (new_unit_cost * new_quantity) - new_discount;

        let mut q = PurchaseOrderItemEntity::update_many()
            .filter(PurchaseOrderItemColumn::Id.eq(item_id))
            .col_expr(PurchaseOrderItemColumn::UpdatedAt, Expr::value(now.clone()))
            .col_expr(PurchaseOrderItemColumn::Quantity, Expr::value(new_quantity))
            .col_expr(
                PurchaseOrderItemColumn::UnitCost,
                Expr::value(new_unit_cost),
            )
            .col_expr(
                PurchaseOrderItemColumn::DiscountAmount,
                Expr::value(new_discount),
            )
            .col_expr(
                PurchaseOrderItemColumn::TotalCost,
                Expr::value(new_total_cost),
            );

        if let Some(ref product_name) = data.product_name {
            q = q.col_expr(
                PurchaseOrderItemColumn::ProductName,
                Expr::value(product_name.clone()),
            );
        }
        if data.variant_name.should_update() {
            q = q.col_expr(
                PurchaseOrderItemColumn::VariantName,
                Expr::value(data.variant_name.to_bind_value()),
            );
        }
        if data.barcode.should_update() {
            q = q.col_expr(
                PurchaseOrderItemColumn::Barcode,
                Expr::value(data.barcode.to_bind_value()),
            );
        }
        if data.metadata.should_update() {
            let metadata_str: Option<String> = match &data.metadata {
                Update::Set(v) => serde_json::to_string(v).ok(),
                Update::Clear => None,
                Update::Unchanged => unreachable!(),
            };
            q = q.col_expr(PurchaseOrderItemColumn::Metadata, Expr::value(metadata_str));
        }

        let result = q.exec(&ctx.db).await?;
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Purchase order item with id {} not found",
                item_id
            )));
        }

        // Recalculate order subtotal from all items
        let cost_delta = new_total_cost - existing.total_cost;
        if cost_delta != 0 {
            let order = PurchaseOrderEntity::find_by_id(existing.purchase_order_id)
                .filter(PurchaseOrderColumn::IsDeleted.eq(false))
                .one(&ctx.db)
                .await?;
            if let Some(order) = order {
                let new_subtotal = order.subtotal + cost_delta;
                let new_total = new_subtotal - order.discount_amount;
                PurchaseOrderEntity::update_many()
                    .filter(PurchaseOrderColumn::Id.eq(existing.purchase_order_id))
                    .col_expr(PurchaseOrderColumn::Subtotal, Expr::value(new_subtotal))
                    .col_expr(PurchaseOrderColumn::TotalAmount, Expr::value(new_total))
                    .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now))
                    .exec(&ctx.db)
                    .await?;
            }
        }

        Ok(())
    }

    async fn delete_item(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        item_id: i64,
    ) -> DomainResult<()> {
        let now = now_str();

        let existing = PurchaseOrderItemEntity::find_by_id(item_id)
            .one(&ctx.db)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Purchase order item with id {} not found", item_id))
            })?;

        let result = PurchaseOrderItemEntity::delete_many()
            .filter(PurchaseOrderItemColumn::Id.eq(item_id))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Purchase order item with id {} not found",
                item_id
            )));
        }

        // Update order subtotal/total
        let order = PurchaseOrderEntity::find_by_id(existing.purchase_order_id)
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;
        if let Some(order) = order {
            let new_subtotal = order.subtotal - existing.total_cost;
            let new_total = new_subtotal - order.discount_amount;
            PurchaseOrderEntity::update_many()
                .filter(PurchaseOrderColumn::Id.eq(existing.purchase_order_id))
                .col_expr(PurchaseOrderColumn::Subtotal, Expr::value(new_subtotal))
                .col_expr(PurchaseOrderColumn::TotalAmount, Expr::value(new_total))
                .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now))
                .exec(&ctx.db)
                .await?;
        }

        Ok(())
    }

    async fn get_items(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
    ) -> DomainResult<Vec<PurchaseOrderItem>> {
        let items = PurchaseOrderItemEntity::find()
            .filter(PurchaseOrderItemColumn::PurchaseOrderId.eq(purchase_order_id))
            .all(&ctx.db)
            .await?;
        Ok(items.into_iter().map(|i| i.to_domain()).collect())
    }

    // ── Payment management ────────────────────────────────────────────────────

    async fn update_payment(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        payment_id: i64,
        data: &PurchasePaymentUpdate,
    ) -> DomainResult<()> {
        let existing = PurchasePaymentEntity::find_by_id(payment_id)
            .one(&ctx.db)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Purchase payment with id {} not found", payment_id))
            })?;

        let mut q =
            PurchasePaymentEntity::update_many().filter(PurchasePaymentColumn::Id.eq(payment_id));

        if let Some(amount) = data.amount {
            q = q.col_expr(PurchasePaymentColumn::Amount, Expr::value(amount));
        }
        if let Some(payment_channel_id) = data.payment_channel_id {
            q = q.col_expr(
                PurchasePaymentColumn::PaymentChannelId,
                Expr::value(payment_channel_id),
            );
        }
        if let Some(paid_at) = &data.paid_at {
            q = q.col_expr(PurchasePaymentColumn::PaidAt, Expr::value(paid_at.clone()));
        }
        if data.reference.should_update() {
            q = q.col_expr(
                PurchasePaymentColumn::Reference,
                Expr::value(data.reference.to_bind_value()),
            );
        }
        if data.notes.should_update() {
            q = q.col_expr(
                PurchasePaymentColumn::Notes,
                Expr::value(data.notes.to_bind_value()),
            );
        }

        let result = q.exec(&ctx.db).await?;
        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Purchase payment with id {} not found",
                payment_id
            )));
        }

        // Recalculate paid_amount and payment_status on the order if amount changed
        if let Some(new_amount) = data.amount {
            let amount_delta = new_amount - existing.amount;
            if amount_delta != 0 {
                let now = now_str();
                let order = PurchaseOrderEntity::find_by_id(existing.purchase_order_id)
                    .filter(PurchaseOrderColumn::IsDeleted.eq(false))
                    .one(&ctx.db)
                    .await?;
                if let Some(order) = order {
                    let new_paid = order.paid_amount + amount_delta;
                    let new_payment_status = if new_paid >= order.total_amount {
                        PaymentStatus::Paid.as_str()
                    } else if new_paid > 0 {
                        PaymentStatus::Partial.as_str()
                    } else {
                        PaymentStatus::Unpaid.as_str()
                    };
                    PurchaseOrderEntity::update_many()
                        .filter(PurchaseOrderColumn::Id.eq(existing.purchase_order_id))
                        .col_expr(PurchaseOrderColumn::PaidAmount, Expr::value(new_paid))
                        .col_expr(
                            PurchaseOrderColumn::PaymentStatus,
                            Expr::value(new_payment_status),
                        )
                        .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now))
                        .exec(&ctx.db)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn delete_payment(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        payment_id: i64,
    ) -> DomainResult<()> {
        let now = now_str();

        let existing = PurchasePaymentEntity::find_by_id(payment_id)
            .one(&ctx.db)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Purchase payment with id {} not found", payment_id))
            })?;

        let result = PurchasePaymentEntity::delete_many()
            .filter(PurchasePaymentColumn::Id.eq(payment_id))
            .exec(&ctx.db)
            .await?;

        if result.rows_affected == 0 {
            return Err(Error::NotFound(format!(
                "Purchase payment with id {} not found",
                payment_id
            )));
        }

        // Recalculate paid_amount and payment_status on the order
        let order = PurchaseOrderEntity::find_by_id(existing.purchase_order_id)
            .filter(PurchaseOrderColumn::IsDeleted.eq(false))
            .one(&ctx.db)
            .await?;
        if let Some(order) = order {
            let new_paid = order.paid_amount - existing.amount;
            let new_payment_status = if new_paid >= order.total_amount {
                PaymentStatus::Paid.as_str()
            } else if new_paid > 0 {
                PaymentStatus::Partial.as_str()
            } else {
                PaymentStatus::Unpaid.as_str()
            };
            PurchaseOrderEntity::update_many()
                .filter(PurchaseOrderColumn::Id.eq(existing.purchase_order_id))
                .col_expr(PurchaseOrderColumn::PaidAmount, Expr::value(new_paid))
                .col_expr(
                    PurchaseOrderColumn::PaymentStatus,
                    Expr::value(new_payment_status),
                )
                .col_expr(PurchaseOrderColumn::UpdatedAt, Expr::value(now))
                .exec(&ctx.db)
                .await?;
        }

        Ok(())
    }

    async fn get_payments(
        &self,
        ctx: &RepoCtx<impl ConnectionTrait>,
        purchase_order_id: i64,
    ) -> DomainResult<Vec<PurchasePayment>> {
        let payments = PurchasePaymentEntity::find()
            .filter(PurchasePaymentColumn::PurchaseOrderId.eq(purchase_order_id))
            .all(&ctx.db)
            .await?;
        Ok(payments.into_iter().map(|p| p.to_domain()).collect())
    }
}
