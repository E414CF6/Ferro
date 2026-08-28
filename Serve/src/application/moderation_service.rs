#![allow(dead_code)]
use crate::domain::errors::DomainError;
use crate::domain::models::{Report, ReportReason, ReportStatus, ReportTargetType};
use crate::domain::repositories::PostRepository;
use crate::infrastructure::db::postgres::Database;
use std::sync::Arc;
use uuid::Uuid;

/// Application Service orchestrating User Reports and Content Moderation
#[derive(Clone)]
pub struct ModerationService {
    db: Arc<Database>,
}

impl ModerationService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn create_report(
        &self,
        reporter_id: Uuid,
        target_type: ReportTargetType,
        target_id: Uuid,
        reason: ReportReason,
        details: Option<String>,
    ) -> Result<Report, DomainError> {
        self.db
            .create_report(reporter_id, target_type, target_id, reason, details)
            .await
    }

    pub async fn resolve_report(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Report, DomainError> {
        self.db.resolve_report(report_id, status).await
    }

    pub async fn get_reports(
        &self,
        status: Option<ReportStatus>,
        limit: Option<usize>,
    ) -> Vec<Report> {
        self.db.get_reports(status, limit).await
    }
}
