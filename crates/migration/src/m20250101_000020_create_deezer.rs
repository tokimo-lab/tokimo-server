use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeezerTracks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeezerTracks::DeezerId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DeezerTracks::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(DeezerTracks::FetchedAt)
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
                    .table(DeezerAlbums::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeezerAlbums::DeezerId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DeezerAlbums::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(DeezerAlbums::FetchedAt)
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
                    .table(DeezerArtists::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeezerArtists::DeezerId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DeezerArtists::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(DeezerArtists::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        // Deezer's docs allow ~50 req/sec; seed a generous bucket.
        let db = manager.get_connection();
        db.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at)
             VALUES ('deezer', 50.0, 50.0, 50.0, now())
             ON CONFLICT (provider) DO UPDATE SET capacity = 50.0, refill_rate_per_sec = 50.0",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM rate_limit_buckets WHERE provider = 'deezer'")
            .await?;
        manager
            .drop_table(Table::drop().table(DeezerArtists::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DeezerAlbums::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DeezerTracks::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum DeezerTracks {
    Table,
    DeezerId,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum DeezerAlbums {
    Table,
    DeezerId,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum DeezerArtists {
    Table,
    DeezerId,
    RawJson,
    FetchedAt,
}
