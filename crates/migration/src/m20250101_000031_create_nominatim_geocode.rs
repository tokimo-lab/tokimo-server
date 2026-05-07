use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NominatimGeocode::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NominatimGeocode::CacheKey)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NominatimGeocode::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(NominatimGeocode::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        // Strict 1 req/s — Nominatim TOS.
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at) \
             VALUES ('nominatim', 1.0, 1.0, 1.0, now()) \
             ON CONFLICT (provider) DO NOTHING",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NominatimGeocode::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NominatimGeocode {
    Table,
    CacheKey,
    RawJson,
    FetchedAt,
}
