use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AssrtSearches::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AssrtSearches::CacheKey).text().not_null().primary_key())
                    .col(ColumnDef::new(AssrtSearches::Query).text().not_null())
                    .col(ColumnDef::new(AssrtSearches::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(AssrtSearches::FetchedAt)
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
                    .table(AssrtSubDetails::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AssrtSubDetails::SubId).text().not_null().primary_key())
                    .col(ColumnDef::new(AssrtSubDetails::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(AssrtSubDetails::FetchedAt)
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
             VALUES ('assrt', 10.0, 10.0, 10.0, now()) \
             ON CONFLICT (provider) DO NOTHING",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AssrtSubDetails::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AssrtSearches::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AssrtSearches {
    Table,
    CacheKey,
    Query,
    RawJson,
    FetchedAt,
}

#[derive(DeriveIden)]
enum AssrtSubDetails {
    Table,
    SubId,
    RawJson,
    FetchedAt,
}
