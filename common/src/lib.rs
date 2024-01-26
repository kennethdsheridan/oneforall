/// The `Logger` trait defines the behavior for async logging messages.
///
/// This logic is added to allow the asynchronous use of the `log_adapter` between the frontend and backend safely.
///
/// # Example
///
/// ```
/// use crate::Logger;
/// use async_trait::async_trait;
///
/// struct ConsoleLogger;
///
/// #[async_trait]
/// impl Logger for ConsoleLogger {
///     async fn log(&self, message: &str) {
///         println!("{}", message);
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let logger = ConsoleLogger;
///     logger.log("Hello, world!").await;
/// }
/// ```
#[async_trait]
pub trait Logger {
    async fn log(&self, message: &str);
    // Add other async methods as needed
}