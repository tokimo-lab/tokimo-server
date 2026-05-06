use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LrclibLyrics::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(LrclibLyrics::CacheKey).text().not_null().primary_key())
                    .col(ColumnDef::new(LrclibLyrics::Artist).text().not_null())
                    .col(ColumnDef::new(LrclibLyrics::Track).text().not_null())
                    .col(ColumnDef::new(LrclibLyrics::Album).text())
                    .col(ColumnDef::new(LrclibLyrics::Duration).integer())
                    .col(ColumnDef::new(LrclibLyrics::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(LrclibLyrics::FetchedAt)
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
            .drop_table(Table::drop().table(LrclibLyrics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LrclibLyrics {
    Table,
    CacheKey,
    Artist,
    Track,
    Album,
    Duration,
    RawJson,
    FetchedAt,
}
