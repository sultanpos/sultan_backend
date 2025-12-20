/// Macro to create a BranchContext with automatic cancellation on request drop.
///
/// This macro creates a cancellation token and BranchContext, then executes the
/// provided async block. If the request is cancelled or dropped, the context's
/// cancellation token is automatically triggered.
///
/// # Example
/// ```rust,ignore
/// async fn handler() -> DomainResult<impl IntoResponse> {
///     with_branch_context!(ctx => {
///         let result = service.some_operation(&ctx).await?;
///         Ok(Json(result))
///     })
/// }
/// ```
#[macro_export]
macro_rules! with_branch_context {
    ($ctx:ident => $body:block) => {{
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let $ctx = sultan_core::domain::context::BranchContext::new_with_cancel_token(
            cancel_token.clone(),
        );

        // Race the operation against request cancellation
        tokio::select! {
            result = async move $body => result,
            _ = cancel_token.cancelled() => {
                Err(sultan_core::domain::Error::Cancelled("Request cancelled".to_string()))
            }
        }
    }};
}
