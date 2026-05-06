use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TmdbObjects::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TmdbObjects::Kind).text().not_null())
                    .col(ColumnDef::new(TmdbObjects::Key).text().not_null())
                    .col(ColumnDef::new(TmdbObjects::RawJson).json_binary().not_null())
                    .col(
                        ColumnDef::new(TmdbObjects::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .primary_key(Index::create().col(TmdbObjects::Kind).col(TmdbObjects::Key))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TmdbObjects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TmdbObjects {
    Table,
    Kind,
    Key,
    RawJson,
    FetchedAt,
}
