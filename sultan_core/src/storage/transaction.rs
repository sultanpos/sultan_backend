use crate::domain::DomainResult;
use async_trait::async_trait;

/// Transaction manager trait for database-agnostic transaction management.
///
/// This trait provides a unified interface for managing database transactions
/// across different database backends (SQLite, PostgreSQL, MySQL, etc.).
///
/// # Design Pattern
///
/// The transaction manager follows the **Unit of Work** pattern, allowing
/// multiple repository operations to be executed atomically within a single
/// transaction context.
///
/// # Architecture
///
/// - **Database-Agnostic**: The trait doesn't expose database-specific types
/// - **Single Responsibility**: Each database implementation (e.g., `SqliteTransactionManager`)
///   manages transactions for that specific database type
/// - **Separation of Concerns**: Transaction management is separate from repository logic
///
/// # Usage
///
/// ```rust,no_run
/// use sultan_core::storage::transaction::TransactionManager;
/// use sultan_core::storage::sqlite::transaction::SqliteTransactionManager;
/// use sultan_core::domain::Context;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let pool = todo!();
/// let tx_manager = SqliteTransactionManager::new(pool);
/// let ctx = Context::new();
///
/// // Begin transaction
/// let mut tx = tx_manager.begin().await?;
///
/// // Perform operations within transaction
/// // ... repository operations using &mut tx ...
///
/// // Commit transaction
/// tx_manager.commit(tx).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// ```rust,no_run
/// # use sultan_core::storage::transaction::TransactionManager;
/// # use sultan_core::storage::sqlite::transaction::SqliteTransactionManager;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let tx_manager: SqliteTransactionManager = todo!();
/// let mut tx = tx_manager.begin().await?;
///
/// match perform_operations(&mut tx).await {
///     Ok(_) => {
///         tx_manager.commit(tx).await?;
///     }
///     Err(e) => {
///         // Rollback on error
///         let _ = tx_manager.rollback(tx).await;
///         return Err(e);
///     }
/// }
/// # Ok(())
/// # }
/// # async fn perform_operations<T>(_tx: &mut T) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
/// ```
///
/// # Implementations
///
/// - **SQLite**: [`SqliteTransactionManager`](crate::storage::sqlite::transaction::SqliteTransactionManager)
/// - **PostgreSQL**: (Future implementation)
/// - **MySQL**: (Future implementation)
///
/// # See Also
///
/// - [`with_transaction!`](crate::with_transaction) - Macro for simplified transaction handling
/// - Repository traits with `*_tx()` methods for transaction-aware operations
#[async_trait]
pub trait TransactionManager: Send + Sync {
    /// The transaction type for this specific database implementation.
    ///
    /// This associated type allows each database backend to use its own
    /// transaction type while maintaining a unified interface.
    ///
    /// # Examples
    ///
    /// - SQLite: `Transaction<'a, Sqlite>`
    /// - PostgreSQL: `Transaction<'a, Postgres>`
    /// - MySQL: `Transaction<'a, MySql>`
    type Transaction<'a>: Send
    where
        Self: 'a;

    /// Begins a new database transaction.
    ///
    /// This method starts a new transaction and returns a transaction handle
    /// that can be used to execute operations within the transaction scope.
    ///
    /// # Returns
    ///
    /// - `Ok(Transaction)` - A transaction handle
    /// - `Err(Error::Database)` - If the transaction could not be started
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The database connection pool is exhausted
    /// - The database is unavailable
    /// - There are underlying database errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sultan_core::storage::transaction::TransactionManager;
    /// # async fn example<T: TransactionManager>(tx_manager: &T) -> Result<(), Box<dyn std::error::Error>> {
    /// let tx = tx_manager.begin().await?;
    /// // Use tx for database operations
    /// # Ok(())
    /// # }
    /// ```
    async fn begin(&self) -> DomainResult<Self::Transaction<'_>>;
    /// Commits the transaction, persisting all changes to the database.
    ///
    /// This method finalizes the transaction and makes all changes permanent.
    /// After a successful commit, all operations performed within the transaction
    /// become visible to other database connections.
    ///
    /// # Parameters
    ///
    /// - `tx` - The transaction to commit (consumes the transaction)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Transaction committed successfully
    /// - `Err(Error::Database)` - If the commit failed
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - There are constraint violations
    /// - There are serialization conflicts (in some isolation levels)
    /// - The database connection was lost
    /// - There are underlying database errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sultan_core::storage::transaction::TransactionManager;
    /// # async fn example<T: TransactionManager>(tx_manager: &T) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tx = tx_manager.begin().await?;
    /// // Perform operations...
    /// tx_manager.commit(tx).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn commit<'a>(&self, tx: Self::Transaction<'a>) -> DomainResult<()>;
    /// Rolls back the transaction, discarding all changes.
    ///
    /// This method cancels the transaction and undoes all operations performed
    /// within the transaction scope. The database state is restored to what it
    /// was before the transaction began.
    ///
    /// # Implicit Rollback
    ///
    /// Note that if a transaction is dropped without calling either `commit()` or
    /// `rollback()`, it will be automatically rolled back. However, explicit
    /// rollback is recommended for error handling clarity.
    ///
    /// # Parameters
    ///
    /// - `tx` - The transaction to rollback (consumes the transaction)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Transaction rolled back successfully
    /// - `Err(Error::Database)` - If the rollback failed
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The database connection was lost
    /// - There are underlying database errors
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sultan_core::storage::transaction::TransactionManager;
    /// # async fn example<T: TransactionManager>(tx_manager: &T) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tx = tx_manager.begin().await?;
    ///
    /// // If an error occurs, rollback the transaction
    /// if let Err(e) = perform_operation(&mut tx).await {
    ///     let _ = tx_manager.rollback(tx).await;
    ///     return Err(e);
    /// }
    ///
    /// tx_manager.commit(tx).await?;
    /// # Ok(())
    /// # }
    /// # async fn perform_operation<T>(_tx: &mut T) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    /// ```
    async fn rollback<'a>(&self, tx: Self::Transaction<'a>) -> DomainResult<()>;
}
