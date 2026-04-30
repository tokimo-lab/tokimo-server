use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cache_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub namespace: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub key: String,
    pub value: Vec<u8>,
    pub expires_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
