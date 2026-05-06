use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SpotifyTokenCache::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SpotifyTokenCache::Id).integer().not_null().primary_key())
                    .col(ColumnDef::new(SpotifyTokenCache::AccessToken).text().not_null())
                    .col(
                        ColumnDef::new(SpotifyTokenCache::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SpotifyArtists::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SpotifyArtists::SpotifyId)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SpotifyArtists::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(SpotifyArtists::FetchedAt)
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
                    .table(SpotifyAlbums::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SpotifyAlbums::SpotifyId).text().not_null().primary_key())
                    .col(ColumnDef::new(SpotifyAlbums::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(SpotifyAlbums::FetchedAt)
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
                    .table(SpotifyTracks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SpotifyTracks::SpotifyId).text().not_null().primary_key())
                    .col(ColumnDef::new(SpotifyTracks::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(SpotifyTracks::FetchedAt)
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
            .drop_table(Table::drop().table(SpotifyTracks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SpotifyAlbums::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SpotifyArtists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SpotifyTokenCache::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum SpotifyTokenCache {
    Table,
    Id,
    AccessToken,
    ExpiresAt,
}

#[derive(DeriveIden)]
enum SpotifyArtists {
    Table,
    SpotifyId,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum SpotifyAlbums {
    Table,
    SpotifyId,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum SpotifyTracks {
    Table,
    SpotifyId,
    RawJson,
    FetchedAt,
}
