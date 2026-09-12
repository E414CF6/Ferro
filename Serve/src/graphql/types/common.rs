use crate::domain::models::MediaType as MediaTypeModel;
use async_graphql::{InputObject, Object};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn encode_cursor(created_at: DateTime<Utc>, id: Uuid) -> String {
    let raw = format!("{}|{}", created_at.to_rfc3339(), id);
    BASE64.encode(raw)
}

pub fn decode_cursor(cursor: &str) -> Option<(DateTime<Utc>, Uuid)> {
    let bytes = BASE64.decode(cursor.trim()).ok()?;
    let s = String::from_utf8(bytes).ok()?;
    let (dt_str, id_str) = s.split_once('|').or_else(|| s.rsplit_once(':'))?;
    let dt = DateTime::parse_from_rfc3339(dt_str)
        .ok()?
        .with_timezone(&Utc);
    let id = Uuid::parse_str(id_str).ok()?;
    Some((dt, id))
}

#[derive(async_graphql::Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum MediaTypeGql {
    Image,
    Video,
    Gif,
}

impl From<MediaTypeModel> for MediaTypeGql {
    fn from(m: MediaTypeModel) -> Self {
        match m {
            MediaTypeModel::Image => MediaTypeGql::Image,
            MediaTypeModel::Video => MediaTypeGql::Video,
            MediaTypeModel::Gif => MediaTypeGql::Gif,
        }
    }
}

impl From<MediaTypeGql> for MediaTypeModel {
    fn from(m: MediaTypeGql) -> Self {
        match m {
            MediaTypeGql::Image => MediaTypeModel::Image,
            MediaTypeGql::Video => MediaTypeModel::Video,
            MediaTypeGql::Gif => MediaTypeModel::Gif,
        }
    }
}

#[derive(InputObject, Clone, Debug)]
pub struct MediaInput {
    pub media_url: String,
    pub media_type: Option<MediaTypeGql>,
    pub alt_text: Option<String>,
    pub sort_order: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

#[derive(InputObject, Clone, Debug)]
pub struct CreatePollInput {
    pub question: String,
    pub options: Vec<String>,
    pub duration_seconds: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct PageInfoGql {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}

#[Object]
impl PageInfoGql {
    async fn has_next_page(&self) -> bool {
        self.has_next_page
    }

    async fn has_previous_page(&self) -> bool {
        self.has_previous_page
    }

    async fn start_cursor(&self) -> Option<&str> {
        self.start_cursor.as_deref()
    }

    async fn end_cursor(&self) -> Option<&str> {
        self.end_cursor.as_deref()
    }
}
