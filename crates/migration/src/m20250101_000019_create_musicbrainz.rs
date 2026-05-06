use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MusicbrainzArtists::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MusicbrainzArtists::Mbid).text().not_null().primary_key())
                    .col(ColumnDef::new(MusicbrainzArtists::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(MusicbrainzArtists::FetchedAt)
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
                    .table(MusicbrainzReleases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MusicbrainzReleases::Mbid)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MusicbrainzReleases::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(MusicbrainzReleases::FetchedAt)
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
                    .table(MusicbrainzRecordings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MusicbrainzRecordings::Mbid)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MusicbrainzRecordings::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(MusicbrainzRecordings::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        // Seed the per-provider rate limit bucket: MusicBrainz TOS requires
        // ≤ 1 req/sec across the whole process, regardless of how many
        // concurrent callers there are. The PgRateLimiter will reuse this
        // row's capacity / refill_rate for all subsequent acquires.
        let db = manager.get_connection();
        db.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at)
             VALUES ('musicbrainz', 1.0, 1.0, 1.0, now())
             ON CONFLICT (provider) DO UPDATE SET capacity = 1.0, refill_rate_per_sec = 1.0",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM rate_limit_buckets WHERE provider = 'musicbrainz'")
            .await?;
        manager
            .drop_table(Table::drop().table(MusicbrainzRecordings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MusicbrainzReleases::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MusicbrainzArtists::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum MusicbrainzArtists {
    Table,
    Mbid,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum MusicbrainzReleases {
    Table,
    Mbid,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum MusicbrainzRecordings {
    Table,
    Mbid,
    RawJson,
    FetchedAt,
}
