use sultan_core::{
    domain::{Context, model::number::NumberGenerateParams},
    storage::{NumberRepository, sqlite::SqliteNumberRepository},
};

mod common;

#[tokio::test]
async fn test_generate_next_creates_new_sequence() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params = NumberGenerateParams {
        prefix: "TEST".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let number = repo.generate_next(&ctx, &params).await;
    assert!(number.is_ok());
    assert_eq!(number.unwrap(), 1);
}

#[tokio::test]
async fn test_generate_next_increments_existing_sequence() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params = NumberGenerateParams {
        prefix: "TEST".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Generate first number
    let first = repo.generate_next(&ctx, &params).await.unwrap();
    assert_eq!(first, 1);

    // Generate second number
    let second = repo.generate_next(&ctx, &params).await.unwrap();
    assert_eq!(second, 2);

    // Generate third number
    let third = repo.generate_next(&ctx, &params).await.unwrap();
    assert_eq!(third, 3);
}

#[tokio::test]
async fn test_generate_next_different_prefixes() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params1 = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params2 = NumberGenerateParams {
        prefix: "SUP".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Different prefixes should have independent sequences
    let cus1 = repo.generate_next(&ctx, &params1).await.unwrap();
    let sup1 = repo.generate_next(&ctx, &params2).await.unwrap();
    let cus2 = repo.generate_next(&ctx, &params1).await.unwrap();

    assert_eq!(cus1, 1);
    assert_eq!(sup1, 1);
    assert_eq!(cus2, 2);
}

#[tokio::test]
async fn test_generate_next_with_branch() {
    let pool = common::setup_test_db().await;

    // Create a test branch
    common::create_test_branch(&pool, 1, "BR01").await;

    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params_global = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params_branch = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: Some(1),
        year: 2025,
        month: None,
    };

    // Global and branch sequences should be independent
    let global1 = repo.generate_next(&ctx, &params_global).await.unwrap();
    let branch1 = repo.generate_next(&ctx, &params_branch).await.unwrap();
    let global2 = repo.generate_next(&ctx, &params_global).await.unwrap();
    let branch2 = repo.generate_next(&ctx, &params_branch).await.unwrap();

    assert_eq!(global1, 1);
    assert_eq!(branch1, 1);
    assert_eq!(global2, 2);
    assert_eq!(branch2, 2);
}

#[tokio::test]
async fn test_generate_next_with_month() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params_no_month = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params_month1 = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2025,
        month: Some(1),
    };

    let params_month2 = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2025,
        month: Some(2),
    };

    // Different months should have independent sequences
    let no_month = repo.generate_next(&ctx, &params_no_month).await.unwrap();
    let month1_first = repo.generate_next(&ctx, &params_month1).await.unwrap();
    let month2_first = repo.generate_next(&ctx, &params_month2).await.unwrap();
    let month1_second = repo.generate_next(&ctx, &params_month1).await.unwrap();

    assert_eq!(no_month, 1);
    assert_eq!(month1_first, 1);
    assert_eq!(month2_first, 1);
    assert_eq!(month1_second, 2);
}

#[tokio::test]
async fn test_generate_next_different_years() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params_2025 = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let params_2026 = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: None,
        year: 2026,
        month: None,
    };

    // Different years should have independent sequences
    let year2025_first = repo.generate_next(&ctx, &params_2025).await.unwrap();
    let year2026_first = repo.generate_next(&ctx, &params_2026).await.unwrap();
    let year2025_second = repo.generate_next(&ctx, &params_2025).await.unwrap();

    assert_eq!(year2025_first, 1);
    assert_eq!(year2026_first, 1);
    assert_eq!(year2025_second, 2);
}

#[tokio::test]
async fn test_get_sequence_returns_none_for_nonexistent() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params = NumberGenerateParams {
        prefix: "NONEXIST".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    let result = repo.get_sequence(&ctx, &params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_get_sequence_returns_existing() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);
    let ctx = Context::new();

    let params = NumberGenerateParams {
        prefix: "TEST".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Generate a number to create the sequence
    repo.generate_next(&ctx, &params).await.unwrap();

    // Now get the sequence
    let result = repo.get_sequence(&ctx, &params).await;
    assert!(result.is_ok());

    let sequence = result.unwrap();
    assert!(sequence.is_some());

    let seq = sequence.unwrap();
    assert_eq!(seq.prefix, "TEST");
    assert_eq!(seq.year, 2025);
    assert_eq!(seq.last_number, 1);
}

#[tokio::test]
async fn test_concurrent_generation_no_duplicates() {
    let pool = common::setup_test_db().await;
    let repo = SqliteNumberRepository::new(pool);

    let params = NumberGenerateParams {
        prefix: "CONC".to_string(),
        branch_id: None,
        year: 2025,
        month: None,
    };

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for _ in 0..10 {
        let repo_clone = repo.clone();
        let params_clone = params.clone();
        let handle = tokio::spawn(async move {
            let ctx = Context::new();
            repo_clone.generate_next(&ctx, &params_clone).await
        });
        handles.push(handle);
    }

    // Collect all results
    let mut numbers = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        numbers.push(result.unwrap());
    }

    // Sort the numbers
    numbers.sort_unstable();

    // Verify no duplicates and sequential
    assert_eq!(numbers.len(), 10);
    for (i, &num) in numbers.iter().enumerate() {
        assert_eq!(num, (i + 1) as i32);
    }
}

#[tokio::test]
async fn test_branch_deletion_cascades() {
    let pool = common::setup_test_db().await;

    // Create a test branch
    let branch_id = common::create_test_branch(&pool, 1, "BR01").await;

    let repo = SqliteNumberRepository::new(pool.clone());
    let ctx = Context::new();

    let params = NumberGenerateParams {
        prefix: "CUS".to_string(),
        branch_id: Some(branch_id),
        year: 2025,
        month: None,
    };

    // Generate a number for this branch
    repo.generate_next(&ctx, &params).await.unwrap();

    // Verify sequence exists
    let seq_before = repo.get_sequence(&ctx, &params).await.unwrap();
    assert!(seq_before.is_some());

    // Delete the branch
    sqlx::query("DELETE FROM branches WHERE id = ?")
        .bind(branch_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify sequence is deleted (cascade)
    let seq_after = repo.get_sequence(&ctx, &params).await.unwrap();
    assert!(seq_after.is_none());
}
