use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lrclib_lyrics")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub cache_key: String,
    pub artist: String,
    pub track: String,
    pub album: Option<String>,
    pub duration: Option<i32>,
    pub raw_json: Json,
    pub fetched_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
