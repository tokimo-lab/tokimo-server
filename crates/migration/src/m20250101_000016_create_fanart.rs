use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FanartAssets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(FanartAssets::Kind).text().not_null())
                    .col(ColumnDef::new(FanartAssets::ForeignId).big_integer().not_null())
                    .col(ColumnDef::new(FanartAssets::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(FanartAssets::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .primary_key(Index::create().col(FanartAssets::Kind).col(FanartAssets::ForeignId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FanartAssets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FanartAssets {
    Table,
    Kind,
    ForeignId,
    RawJson,
    FetchedAt,
}
