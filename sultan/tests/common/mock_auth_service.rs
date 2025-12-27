use async_trait::async_trait;
use sultan_core::application::{AuthServiceTrait, AuthTokens};
use sultan_core::domain::{DomainResult, Error, context::BranchContext};

pub struct MockAuthService {
    pub should_succeed: bool,
    pub access_token: String,
    pub refresh_token: String,
}

impl MockAuthService {
    pub fn new_success() -> Self {
        Self {
            should_succeed: true,
            access_token: "mock_access_token_12345".to_string(),
            refresh_token: "mock_refresh_token_67890".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn new_failure() -> Self {
        Self {
            should_succeed: false,
            access_token: String::new(),
            refresh_token: String::new(),
        }
    }
}

#[async_trait]
impl AuthServiceTrait<BranchContext> for MockAuthService {
    async fn login(
        &self,
        _ctx: &BranchContext,
        username: &str,
        password: &str,
    ) -> DomainResult<AuthTokens> {
        if !self.should_succeed {
            return Err(Error::InvalidCredentials);
        }

        // Mock logic: accept testuser/testpassword123
        if username == "testuser" && password == "testpassword123" {
            Ok(AuthTokens {
                access_token: self.access_token.clone(),
                refresh_token: self.refresh_token.clone(),
            })
        } else {
            Err(Error::InvalidCredentials)
        }
    }

    async fn refresh(
        &self,
        _ctx: &BranchContext,
        _refresh_token: &str,
    ) -> DomainResult<AuthTokens> {
        if !self.should_succeed {
            return Err(Error::Unauthorized("Invalid refresh token".to_string()));
        }

        Ok(AuthTokens {
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
        })
    }

    async fn logout(&self, _ctx: &BranchContext, _refresh_token: &str) -> DomainResult<()> {
        if !self.should_succeed {
            return Err(Error::Unauthorized("Invalid refresh token".to_string()));
        }
        Ok(())
    }
}
