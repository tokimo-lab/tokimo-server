use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "rate_limit_buckets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub provider: String,
    pub tokens: f64,
    pub capacity: f64,
    pub refill_rate_per_sec: f64,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
