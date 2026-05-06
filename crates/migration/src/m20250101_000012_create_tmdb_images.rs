use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TmdbImages::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TmdbImages::ImagePath).text().not_null().primary_key())
                    .col(ColumnDef::new(TmdbImages::StorageKey).text().not_null())
                    .col(ColumnDef::new(TmdbImages::Sha256).text().not_null())
                    .col(
                        ColumnDef::new(TmdbImages::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TmdbImages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TmdbImages {
    Table,
    ImagePath,
    StorageKey,
    Sha256,
    FetchedAt,
}
