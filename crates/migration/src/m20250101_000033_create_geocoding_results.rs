use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GeocodingResults::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GeocodingResults::CacheKey)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GeocodingResults::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(GeocodingResults::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        // Aggregator routes to per-source rate limiters too; this is the
        // outer envelope for the aggregator endpoint itself.
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at) \
             VALUES ('geocoding', 30.0, 30.0, 30.0, now()) \
             ON CONFLICT (provider) DO NOTHING",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GeocodingResults::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GeocodingResults {
    Table,
    CacheKey,
    RawJson,
    FetchedAt,
}
