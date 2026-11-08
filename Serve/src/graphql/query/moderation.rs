use crate::application::AppServices;
use crate::application::helpers::require_auth;
use crate::domain::models::ReportStatus;
use crate::graphql::types::{ReportGql, ReportStatusGql};
use async_graphql::{Context, ErrorExtensions, Object, Result};

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
        let services = ctx.data::<AppServices>()?;
        let st_model = status.map(ReportStatus::from);
        let list = services
            .moderation
            .get_reports(st_model, limit)
            .await
            .map_err(|e| e.extend())?;
        Ok(list.into_iter().map(ReportGql).collect())
    }
}
