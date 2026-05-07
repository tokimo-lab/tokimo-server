use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OpenmeteoForecasts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OpenmeteoForecasts::CacheKey)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OpenmeteoForecasts::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(OpenmeteoForecasts::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();
        conn.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at) \
             VALUES ('openmeteo', 100.0, 100.0, 100.0, now()) \
             ON CONFLICT (provider) DO NOTHING",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OpenmeteoForecasts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OpenmeteoForecasts {
    Table,
    CacheKey,
    RawJson,
    FetchedAt,
}
