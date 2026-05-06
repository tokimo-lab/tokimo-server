use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "fanart_assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub kind: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub foreign_id: i64,
    pub raw_json: Json,
    pub fetched_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
