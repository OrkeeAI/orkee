#[cfg(test)]
mod protocol_tests;

#[cfg(test)]
mod tool_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
pub mod test_helpers {
    use crate::context::ToolContext;

    /// Create a test context with isolated in-memory storage
    /// Each test gets its own isolated database that doesn't interfere with other tests
    pub async fn create_test_context(
    ) -> Result<ToolContext, Box<dyn std::error::Error + Send + Sync>> {
        crate::context::test_utils::create_test_context().await
    }
}
