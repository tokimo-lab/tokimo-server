use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HolidayYears::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(HolidayYears::Country).text().not_null())
                    .col(ColumnDef::new(HolidayYears::Year).integer().not_null())
                    .col(ColumnDef::new(HolidayYears::Source).text().not_null())
                    .col(ColumnDef::new(HolidayYears::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(HolidayYears::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .primary_key(Index::create().col(HolidayYears::Country).col(HolidayYears::Year))
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();
        conn.execute_unprepared(
            "INSERT INTO rate_limit_buckets (provider, tokens, capacity, refill_rate_per_sec, updated_at) \
             VALUES ('holiday', 10.0, 10.0, 10.0, now()) \
             ON CONFLICT (provider) DO NOTHING",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HolidayYears::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HolidayYears {
    Table,
    Country,
    Year,
    Source,
    RawJson,
    FetchedAt,
}
