use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CurrencyRates::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CurrencyRates::Base).text().not_null())
                    .col(ColumnDef::new(CurrencyRates::TargetsCsv).text().not_null())
                    .col(ColumnDef::new(CurrencyRates::Days).integer().not_null())
                    .col(ColumnDef::new(CurrencyRates::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(CurrencyRates::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .primary_key(
                        Index::create()
                            .col(CurrencyRates::Base)
                            .col(CurrencyRates::TargetsCsv)
                            .col(CurrencyRates::Days),
                    )
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();
        conn.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at) \
             VALUES ('currency', 10.0, 10.0, 10.0, now()) \
             ON CONFLICT (provider) DO NOTHING",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CurrencyRates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CurrencyRates {
    Table,
    Base,
    TargetsCsv,
    Days,
    RawJson,
    FetchedAt,
}
