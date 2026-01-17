use async_trait::async_trait;

use crate::domain::{
    Context, DomainResult,
    model::number::{NumberGenerateParams, NumberSequence},
};

/// Repository trait for number sequence management
///
/// This trait provides methods for generating sequential numbers with support for:
/// - Global numbering (when branch_id is None)
/// - Branch-specific numbering (when branch_id is Some)
/// - Optional month-based segmentation (when month is Some)
///
/// Implementations must ensure thread-safety and prevent duplicate number generation
/// under concurrent access.
#[async_trait]
pub trait NumberRepository: Send + Sync {
    /// Generate the next number for the given parameters
    ///
    /// This method must be atomic and thread-safe. It should:
    /// 1. Find or create the sequence record for the given parameters
    /// 2. Increment the last_number
    /// 3. Return the new number
    ///
    /// # Arguments
    /// * `ctx` - The execution context
    /// * `params` - Parameters defining the number sequence (prefix, branch_id, year, month)
    ///
    /// # Returns
    /// The next sequential number for this sequence
    async fn generate_next(
        &self,
        ctx: &Context,
        params: &NumberGenerateParams,
    ) -> DomainResult<i32>;

    /// Get the current sequence record for the given parameters
    ///
    /// # Arguments
    /// * `ctx` - The execution context
    /// * `params` - Parameters defining the number sequence
    ///
    /// # Returns
    /// The sequence record if it exists, None otherwise
    async fn get_sequence(
        &self,
        ctx: &Context,
        params: &NumberGenerateParams,
    ) -> DomainResult<Option<NumberSequence>>;
}
