use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Assets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Assets::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(ColumnDef::new(Assets::SourceUrl).text().not_null())
                    .col(ColumnDef::new(Assets::StorageKey).text().not_null().unique_key())
                    .col(ColumnDef::new(Assets::Mime).text().not_null())
                    .col(ColumnDef::new(Assets::Size).big_integer().not_null())
                    .col(ColumnDef::new(Assets::Sha256).text().not_null().unique_key())
                    .col(ColumnDef::new(Assets::Provider).text().not_null())
                    .col(
                        ColumnDef::new(Assets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_assets_source_url")
                    .table(Assets::Table)
                    .col(Assets::SourceUrl)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Assets::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Assets {
    Table,
    Id,
    SourceUrl,
    StorageKey,
    Mime,
    Size,
    Sha256,
    Provider,
    CreatedAt,
}
