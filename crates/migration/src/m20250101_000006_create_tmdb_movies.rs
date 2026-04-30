use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TmdbMovies::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TmdbMovies::Id).integer().not_null().primary_key())
                    .col(ColumnDef::new(TmdbMovies::Title).text().not_null())
                    .col(ColumnDef::new(TmdbMovies::OriginalTitle).text())
                    .col(ColumnDef::new(TmdbMovies::Overview).text())
                    .col(ColumnDef::new(TmdbMovies::ReleaseDate).text())
                    .col(ColumnDef::new(TmdbMovies::Runtime).integer())
                    .col(ColumnDef::new(TmdbMovies::VoteAverage).double())
                    .col(ColumnDef::new(TmdbMovies::VoteCount).integer())
                    .col(ColumnDef::new(TmdbMovies::PosterStorageKey).text())
                    .col(ColumnDef::new(TmdbMovies::BackdropStorageKey).text())
                    .col(ColumnDef::new(TmdbMovies::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(TmdbMovies::FetchedAt)
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
            .drop_table(Table::drop().table(TmdbMovies::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TmdbMovies {
    Table,
    Id,
    Title,
    OriginalTitle,
    Overview,
    ReleaseDate,
    Runtime,
    VoteAverage,
    VoteCount,
    PosterStorageKey,
    BackdropStorageKey,
    RawJson,
    FetchedAt,
}
