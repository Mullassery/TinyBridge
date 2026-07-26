pub mod guest;
pub mod network;
pub mod resources;
pub mod storage;
pub mod virtualization;

use crate::result::CheckResult;
use async_trait::async_trait;

#[async_trait]
pub trait DiagnosticCheck: Send + Sync {
    async fn run(&self) -> CheckResult;
}
