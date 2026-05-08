use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OpensubtitlesCache::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OpensubtitlesCache::CacheKey)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OpensubtitlesCache::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(OpensubtitlesCache::FetchedAt)
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
            .drop_table(Table::drop().table(OpensubtitlesCache::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OpensubtitlesCache {
    Table,
    CacheKey,
    RawJson,
    FetchedAt,
}
