use async_trait::async_trait;
use sea_orm::ConnectionTrait;

use crate::domain::{
    DomainResult,
    model::number::{NumberGenerateParams, NumberSequence},
};

/// Repository trait for number sequence management.
///
/// This trait provides methods for generating sequential numbers with support for:
/// - Global numbering (when branch_id is None)
/// - Branch-specific numbering (when branch_id is Some)
/// - Optional month-based segmentation (when month is Some)
///
/// All methods accept `RepoCtx<impl ConnectionTrait>` to support both direct database
/// access and transactional operations.
///
/// Implementations must ensure thread-safety and prevent duplicate number generation
/// under concurrent access.
///
/// # Implementations
///
/// - SQLite: [`SqliteNumberRepository`](crate::storage::sqlite::number::SqliteNumberRepository)
///
/// # Example
///
/// ```rust,ignore
/// use sultan_core::storage::number_repo::{NumberRepository, RepoCtx};
/// use sultan_core::storage::sqlite::number::SqliteNumberRepository;
///
/// async fn example(db: &DatabaseConnection) -> DomainResult<()> {
///     let repo = SqliteNumberRepository::new();
///     let ctx = RepoCtx {
///         ctx: Context::new(),
///         db,
///     };
///     
///     // Define parameters for number generation
///     let params = NumberGenerateParams {
///         prefix: "CUS".to_string(),
///         branch_id: Some(1),
///         year: 2025,
///         month: Some(1),
///     };
///     
///     // Generate sequential numbers
///     let first = repo.generate_next(&ctx, &params).await?;  // Returns 1
///     let second = repo.generate_next(&ctx, &params).await?; // Returns 2
///     
///     // Get the current sequence state
///     let sequence = repo.get_sequence(&ctx, &params).await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait NumberRepository: Send + Sync {
    /// Generates the next number for the given parameters.
    ///
    /// This method must be atomic and thread-safe. It will:
    /// 1. Find or create the sequence record for the given parameters
    /// 2. Increment the last_number
    /// 3. Return the new number
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `params` - Parameters defining the number sequence (prefix, branch_id, year, month)
    ///
    /// # Returns
    ///
    /// * `Ok(i32)` - The next sequential number for this sequence
    /// * `Err(Error)` - Database error
    async fn generate_next(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        params: &NumberGenerateParams,
    ) -> DomainResult<i32>;

    /// Gets the current sequence record for the given parameters.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Repository context with database connection
    /// * `params` - Parameters defining the number sequence
    ///
    /// # Returns
    ///
    /// * `Ok(Some(sequence))` - The sequence record if it exists
    /// * `Ok(None)` - Sequence not found
    /// * `Err(Error)` - Database error
    async fn get_sequence(
        &self,
        ctx: &super::RepoCtx<impl ConnectionTrait>,
        params: &NumberGenerateParams,
    ) -> DomainResult<Option<NumberSequence>>;
}
