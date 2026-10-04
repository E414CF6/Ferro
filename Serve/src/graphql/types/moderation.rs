use async_graphql::{Context, ErrorExtensions, ID, Object, Result};
use chrono::{DateTime, Utc};

use crate::domain::errors::{DomainError, ErrorCode};
use crate::domain::models::{
    Report as ReportModel, ReportReason as ReportReasonModel, ReportStatus as ReportStatusModel,
    ReportTargetType as ReportTargetTypeModel,
};
use crate::domain::repositories::*;
use crate::infrastructure::db::postgres::Database;

use super::UserGql;

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReportTargetTypeGql {
    Post,
    Comment,
    User,
}

impl From<ReportTargetTypeModel> for ReportTargetTypeGql {
    fn from(r: ReportTargetTypeModel) -> Self {
        match r {
            ReportTargetTypeModel::Post => ReportTargetTypeGql::Post,
            ReportTargetTypeModel::Comment => ReportTargetTypeGql::Comment,
            ReportTargetTypeModel::User => ReportTargetTypeGql::User,
        }
    }
}

impl From<ReportTargetTypeGql> for ReportTargetTypeModel {
    fn from(r: ReportTargetTypeGql) -> Self {
        match r {
            ReportTargetTypeGql::Post => ReportTargetTypeModel::Post,
            ReportTargetTypeGql::Comment => ReportTargetTypeModel::Comment,
            ReportTargetTypeGql::User => ReportTargetTypeModel::User,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReportReasonGql {
    Spam,
    Harassment,
    HateSpeech,
    Inappropriate,
    Copyright,
    Other,
}

impl From<ReportReasonModel> for ReportReasonGql {
    fn from(r: ReportReasonModel) -> Self {
        match r {
            ReportReasonModel::Spam => ReportReasonGql::Spam,
            ReportReasonModel::Harassment => ReportReasonGql::Harassment,
            ReportReasonModel::HateSpeech => ReportReasonGql::HateSpeech,
            ReportReasonModel::Inappropriate => ReportReasonGql::Inappropriate,
            ReportReasonModel::Copyright => ReportReasonGql::Copyright,
            ReportReasonModel::Other => ReportReasonGql::Other,
        }
    }
}

impl From<ReportReasonGql> for ReportReasonModel {
    fn from(r: ReportReasonGql) -> Self {
        match r {
            ReportReasonGql::Spam => ReportReasonModel::Spam,
            ReportReasonGql::Harassment => ReportReasonModel::Harassment,
            ReportReasonGql::HateSpeech => ReportReasonModel::HateSpeech,
            ReportReasonGql::Inappropriate => ReportReasonModel::Inappropriate,
            ReportReasonGql::Copyright => ReportReasonModel::Copyright,
            ReportReasonGql::Other => ReportReasonModel::Other,
        }
    }
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReportStatusGql {
    Pending,
    Resolved,
    Dismissed,
}

impl From<ReportStatusModel> for ReportStatusGql {
    fn from(s: ReportStatusModel) -> Self {
        match s {
            ReportStatusModel::Pending => ReportStatusGql::Pending,
            ReportStatusModel::Resolved => ReportStatusGql::Resolved,
            ReportStatusModel::Dismissed => ReportStatusGql::Dismissed,
        }
    }
}

impl From<ReportStatusGql> for ReportStatusModel {
    fn from(s: ReportStatusGql) -> Self {
        match s {
            ReportStatusGql::Pending => ReportStatusModel::Pending,
            ReportStatusGql::Resolved => ReportStatusModel::Resolved,
            ReportStatusGql::Dismissed => ReportStatusModel::Dismissed,
        }
    }
}

#[derive(Clone)]
pub struct ReportGql(pub ReportModel);

#[Object]
impl ReportGql {
    async fn id(&self) -> ID {
        ID(self.0.id.to_string())
    }

    async fn target_id(&self) -> ID {
        ID(self.0.target_id.to_string())
    }

    async fn target_type(&self) -> ReportTargetTypeGql {
        ReportTargetTypeGql::from(self.0.target_type)
    }

    async fn reason(&self) -> ReportReasonGql {
        ReportReasonGql::from(self.0.reason)
    }

    async fn details(&self) -> Option<&str> {
        self.0.details.as_deref()
    }

    async fn status(&self) -> ReportStatusGql {
        ReportStatusGql::from(self.0.status)
    }

    async fn created_at(&self) -> DateTime<Utc> {
        self.0.created_at
    }

    async fn reporter(&self, ctx: &Context<'_>) -> Result<UserGql> {
        let db = ctx.data::<Database>()?;
        let user = db.get_user_by_id(self.0.reporter_id).await.ok_or_else(|| {
            DomainError::new(ErrorCode::UserNotFound, "Reporter not found").extend()
        })?;
        Ok(UserGql(user))
    }
}
