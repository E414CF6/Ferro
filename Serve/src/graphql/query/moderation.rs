use crate::application::helpers::require_auth;
use crate::domain::models::ReportStatus;
use crate::domain::repositories::ModerationRepository;
use crate::graphql::types::{ReportGql, ReportStatusGql};
use crate::infrastructure::db::postgres::Database;
use async_graphql::{Context, Object, Result};

#[derive(Default)]
pub struct ModerationQuery;

#[Object]
impl ModerationQuery {
    /// Get content reports list (moderation query)
    async fn reports(
        &self,
        ctx: &Context<'_>,
        status: Option<ReportStatusGql>,
        limit: Option<usize>,
    ) -> Result<Vec<ReportGql>> {
        let _ = require_auth(ctx)?;
        let db = ctx.data::<Database>()?;
        let st_model = status.map(ReportStatus::from);
        let list = db.get_reports(st_model, limit).await;
        Ok(list.into_iter().map(ReportGql).collect())
    }
}
