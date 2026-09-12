use crate::domain::errors::DomainError;
use crate::domain::models::{Report, ReportReason, ReportStatus, ReportTargetType};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
#[allow(dead_code)]
pub trait ModerationRepository: Send + Sync {
    async fn create_report(
        &self,
        reporter_id: Uuid,
        target_type: ReportTargetType,
        target_id: Uuid,
        reason: ReportReason,
        details: Option<String>,
    ) -> Result<Report, DomainError>;
    async fn resolve_report(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Report, DomainError>;
    async fn get_reports(&self, status: Option<ReportStatus>, limit: Option<usize>) -> Vec<Report>;
}
