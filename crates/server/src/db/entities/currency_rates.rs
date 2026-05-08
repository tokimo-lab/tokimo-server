use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "currency_rates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub base: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub targets_csv: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub days: i32,
    pub raw_json: Json,
    pub fetched_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
