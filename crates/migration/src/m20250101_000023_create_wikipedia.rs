use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WikipediaSummaries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WikipediaSummaries::CacheKey)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WikipediaSummaries::Lang).text().not_null())
                    .col(ColumnDef::new(WikipediaSummaries::Title).text().not_null())
                    .col(ColumnDef::new(WikipediaSummaries::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(WikipediaSummaries::FetchedAt)
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
            .drop_table(Table::drop().table(WikipediaSummaries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum WikipediaSummaries {
    Table,
    CacheKey,
    Lang,
    Title,
    RawJson,
    FetchedAt,
}
