use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BangumiSubjects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BangumiSubjects::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BangumiSubjects::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(BangumiSubjects::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BangumiSubjects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BangumiSubjects {
    Table,
    Id,
    RawJson,
    FetchedAt,
}
