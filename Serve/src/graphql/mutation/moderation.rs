use crate::application::helpers::resolve_user_id;
use crate::domain::models::{ReportReason, ReportStatus, ReportTargetType};
use crate::domain::repositories::ModerationRepository;
use crate::graphql::types::{ReportGql, ReportReasonGql, ReportStatusGql, ReportTargetTypeGql};
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use uuid::Uuid;

#[derive(Default)]
pub struct ModerationMutation;

#[Object]
impl ModerationMutation {
    /// Report a post, comment, or user for moderation
    async fn report_content(
        &self,
        ctx: &Context<'_>,
        target_type: ReportTargetTypeGql,
        target_id: ID,
        reason: ReportReasonGql,
        details: Option<String>,
        reporter_id: Option<ID>,
    ) -> Result<ReportGql> {
        let rid = resolve_user_id(ctx, reporter_id)?;
        let tid = Uuid::parse_str(&target_id)?;
        let db = ctx.data::<Database>()?;
        let report = db
            .create_report(
                rid,
                ReportTargetType::from(target_type),
                tid,
                ReportReason::from(reason),
                details,
            )
            .await
            .map_err(|e| e.extend())?;
        Ok(ReportGql(report))
    }

    /// Resolve a reported item (moderation action)
    async fn resolve_report(
        &self,
        ctx: &Context<'_>,
        report_id: ID,
        status: ReportStatusGql,
    ) -> Result<ReportGql> {
        let rid = Uuid::parse_str(&report_id)?;
        let db = ctx.data::<Database>()?;
        let report = db
            .resolve_report(rid, ReportStatus::from(status))
            .await
            .map_err(|e| e.extend())?;
        Ok(ReportGql(report))
    }
}
