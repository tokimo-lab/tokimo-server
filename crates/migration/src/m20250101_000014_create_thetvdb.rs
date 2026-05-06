use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ThetvdbTokenCache::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ThetvdbTokenCache::Id).integer().not_null().primary_key())
                    .col(ColumnDef::new(ThetvdbTokenCache::Token).text().not_null())
                    .col(
                        ColumnDef::new(ThetvdbTokenCache::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ThetvdbSeries::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ThetvdbSeries::Id).big_integer().not_null().primary_key())
                    .col(ColumnDef::new(ThetvdbSeries::RawJson).json_binary().not_null())
                    .col(ColumnDef::new(ThetvdbSeries::EpisodesRawJson).json_binary())
                    .col(ColumnDef::new(ThetvdbSeries::EpisodesFetchedAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(ThetvdbSeries::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ThetvdbEpisodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ThetvdbEpisodes::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ThetvdbEpisodes::SeriesId).big_integer())
                    .col(ColumnDef::new(ThetvdbEpisodes::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(ThetvdbEpisodes::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ThetvdbEpisodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ThetvdbSeries::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ThetvdbTokenCache::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ThetvdbTokenCache {
    Table,
    Id,
    Token,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum ThetvdbSeries {
    Table,
    Id,
    RawJson,
    EpisodesRawJson,
    EpisodesFetchedAt,
    FetchedAt,
}

#[derive(DeriveIden)]
enum ThetvdbEpisodes {
    Table,
    Id,
    SeriesId,
    RawJson,
    FetchedAt,
}
