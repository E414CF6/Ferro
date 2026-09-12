use super::database::Database;
use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{Report, ReportReason, ReportStatus, ReportTargetType};
use crate::domain::repositories::ModerationRepository;
use crate::infrastructure::db::entities::ReportEntity;

use async_trait::async_trait;
use chrono::Utc;
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl ModerationRepository for Database {
    async fn create_report(
        &self,
        reporter_id: Uuid,
        target_type: ReportTargetType,
        target_id: Uuid,
        reason: ReportReason,
        details: Option<String>,
    ) -> Result<Report, DomainError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        db_execute!(
            self,
            "INSERT INTO reports (id, reporter_id, target_type, target_id, reason, details, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            id,
            reporter_id,
            target_type.as_str(),
            target_id,
            reason.as_str(),
            &details,
            ReportStatus::Pending.as_str(),
            now,
            now
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to create report");
            DomainError::new(ErrorCode::ReportNotFound, "Failed to create report")
        })?;

        Ok(Report {
            id,
            reporter_id,
            target_type,
            target_id,
            reason,
            details,
            status: ReportStatus::Pending,
            created_at: now,
            updated_at: now,
        })
    }

    async fn resolve_report(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Report, DomainError> {
        let now = Utc::now();
        let res = db_execute!(
            self,
            "UPDATE reports SET status = $1, updated_at = $2 WHERE id = $3",
            status.as_str(),
            now,
            report_id
        )
        .map_err(|e| {
            error!(target: "serve::db", error = %e, "Failed to resolve report");
            DomainError::new(ErrorCode::ReportNotFound, "Failed to resolve report")
        })?;

        if res.rows_affected() == 0 {
            return Err(DomainError::new(
                ErrorCode::ReportNotFound,
                "Report not found",
            ));
        }

        let entity: Option<ReportEntity> = db_fetch_optional!(
            self,
            ReportEntity,
            "SELECT * FROM reports WHERE id = $1",
            report_id
        )
        .ok()
        .flatten();

        entity
            .map(Report::from)
            .ok_or_else(|| DomainError::new(ErrorCode::ReportNotFound, "Report not found"))
    }

    async fn get_reports(&self, status: Option<ReportStatus>, limit: Option<usize>) -> Vec<Report> {
        let lim = limit.unwrap_or(50) as i64;
        let entities: Vec<ReportEntity> = match status {
            Some(st) => db_fetch_all!(
                self,
                ReportEntity,
                "SELECT * FROM reports WHERE status = $1 ORDER BY created_at DESC LIMIT $2",
                st.as_str(),
                lim
            ),
            None => db_fetch_all!(
                self,
                ReportEntity,
                "SELECT * FROM reports ORDER BY created_at DESC LIMIT $1",
                lim
            ),
        }
        .unwrap_or_default();

        entities.into_iter().map(Report::from).collect()
    }
}
