use sea_orm::DatabaseConnection;

use crate::{
    domain::model::number::NumberGenerateParams,
    storage::{NumberRepository, RepoCtx},
};

/// Runs all NumberRepository tests using the provided repository and context factory.
///
/// # Arguments
///
/// * `repo` - The repository implementation to test
/// * `ctx_factory` - A factory function that creates a fresh RepoCtx for each test
pub async fn number_test_all<C, F, Fut>(repo: &C, ctx_factory: F)
where
    C: NumberRepository,
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = RepoCtx<DatabaseConnection>>,
{
    number_test_generate_next_creates_new_sequence(&ctx_factory().await, repo).await;
    number_test_generate_next_increments_existing(&ctx_factory().await, repo).await;
    number_test_different_prefixes(&ctx_factory().await, repo).await;
    number_test_with_branch(&ctx_factory().await, repo).await;
    number_test_with_month(&ctx_factory().await, repo).await;
    number_test_different_years(&ctx_factory().await, repo).await;
    number_test_get_sequence_nonexistent(&ctx_factory().await, repo).await;
    number_test_get_sequence_existing(&ctx_factory().await, repo).await;
}

/// Test: Generate next creates a new sequence with initial value 1
pub async fn number_test_generate_next_creates_new_sequence<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let params = NumberGenerateParams {
        prefix: format!("TEST_{}", super::generate_test_id().await),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let number = repo.generate_next(ctx, &params).await;
    assert!(number.is_ok());
    assert_eq!(number.unwrap(), 1);
}

/// Test: Generate next increments existing sequence
pub async fn number_test_generate_next_increments_existing<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let params = NumberGenerateParams {
        prefix: format!("INC_{}", super::generate_test_id().await),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Generate first number
    let first = repo.generate_next(ctx, &params).await.unwrap();
    assert_eq!(first, 1);

    // Generate second number
    let second = repo.generate_next(ctx, &params).await.unwrap();
    assert_eq!(second, 2);

    // Generate third number
    let third = repo.generate_next(ctx, &params).await.unwrap();
    assert_eq!(third, 3);
}

/// Test: Different prefixes have independent sequences
pub async fn number_test_different_prefixes<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let unique_id = super::generate_test_id().await;

    let params1 = NumberGenerateParams {
        prefix: format!("CUS_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params2 = NumberGenerateParams {
        prefix: format!("SUP_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Different prefixes should have independent sequences
    let cus1 = repo.generate_next(ctx, &params1).await.unwrap();
    let sup1 = repo.generate_next(ctx, &params2).await.unwrap();
    let cus2 = repo.generate_next(ctx, &params1).await.unwrap();

    assert_eq!(cus1, 1);
    assert_eq!(sup1, 1);
    assert_eq!(cus2, 2);
}

/// Test: Global and branch sequences are independent
pub async fn number_test_with_branch<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    // Create a test branch first
    let branch_id = create_test_branch(ctx, super::generate_test_id().await).await;
    let unique_id = super::generate_test_id().await;

    let params_global = NumberGenerateParams {
        prefix: format!("BR_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params_branch = NumberGenerateParams {
        prefix: format!("BR_{}", unique_id),
        branch_id: Some(branch_id),
        year: 2025,
        month: None,
    };

    // Global and branch sequences should be independent
    let global1 = repo.generate_next(ctx, &params_global).await.unwrap();
    let branch1 = repo.generate_next(ctx, &params_branch).await.unwrap();
    let global2 = repo.generate_next(ctx, &params_global).await.unwrap();
    let branch2 = repo.generate_next(ctx, &params_branch).await.unwrap();

    assert_eq!(global1, 1);
    assert_eq!(branch1, 1);
    assert_eq!(global2, 2);
    assert_eq!(branch2, 2);
}

/// Test: Different months have independent sequences
pub async fn number_test_with_month<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let unique_id = super::generate_test_id().await;

    let params_no_month = NumberGenerateParams {
        prefix: format!("MON_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params_month1 = NumberGenerateParams {
        prefix: format!("MON_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: Some(1),
    };

    let params_month2 = NumberGenerateParams {
        prefix: format!("MON_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: Some(2),
    };

    // Different months should have independent sequences
    let no_month = repo.generate_next(ctx, &params_no_month).await.unwrap();
    let month1_first = repo.generate_next(ctx, &params_month1).await.unwrap();
    let month2_first = repo.generate_next(ctx, &params_month2).await.unwrap();
    let month1_second = repo.generate_next(ctx, &params_month1).await.unwrap();

    assert_eq!(no_month, 1);
    assert_eq!(month1_first, 1);
    assert_eq!(month2_first, 1);
    assert_eq!(month1_second, 2);
}

/// Test: Different years have independent sequences
pub async fn number_test_different_years<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let unique_id = super::generate_test_id().await;

    let params_2025 = NumberGenerateParams {
        prefix: format!("YR_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params_2026 = NumberGenerateParams {
        prefix: format!("YR_{}", unique_id),
        branch_id: None,
        year: 2026,
        month: None,
    };

    // Different years should have independent sequences
    let year2025_first = repo.generate_next(ctx, &params_2025).await.unwrap();
    let year2026_first = repo.generate_next(ctx, &params_2026).await.unwrap();
    let year2025_second = repo.generate_next(ctx, &params_2025).await.unwrap();

    assert_eq!(year2025_first, 1);
    assert_eq!(year2026_first, 1);
    assert_eq!(year2025_second, 2);
}

/// Test: Get sequence returns None for nonexistent sequence
pub async fn number_test_get_sequence_nonexistent<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let params = NumberGenerateParams {
        prefix: format!("NONEXIST_{}", super::generate_test_id().await),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let result = repo.get_sequence(ctx, &params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

/// Test: Get sequence returns existing sequence
pub async fn number_test_get_sequence_existing<R: NumberRepository>(
    ctx: &RepoCtx<DatabaseConnection>,
    repo: &R,
) {
    let unique_id = super::generate_test_id().await;
    let params = NumberGenerateParams {
        prefix: format!("EXIST_{}", unique_id),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Generate a number to create the sequence
    repo.generate_next(ctx, &params).await.unwrap();

    // Now get the sequence
    let result = repo.get_sequence(ctx, &params).await;
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert!(sequence.is_some());

    let seq = sequence.unwrap();
    assert_eq!(seq.prefix, format!("EXIST_{}", unique_id));
    assert_eq!(seq.year, 2025);
    assert_eq!(seq.last_number, 1);
}

/// Helper function to create a test branch for branch-specific number tests
async fn create_test_branch(ctx: &RepoCtx<DatabaseConnection>, id: i64) -> i64 {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

    let code = format!("BR{}", id % 10000);
    let now = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.fZ")
        .to_string();

    ctx.db
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            r#"INSERT INTO branches (id, created_at, updated_at, is_deleted, is_main, name, code) 
               VALUES (?, ?, ?, 0, 0, ?, ?)"#,
            vec![
                id.into(),
                now.clone().into(),
                now.into(),
                format!("Test Branch {}", id).into(),
                code.into(),
            ],
        ))
        .await
        .expect("Failed to create test branch");

    id
}
