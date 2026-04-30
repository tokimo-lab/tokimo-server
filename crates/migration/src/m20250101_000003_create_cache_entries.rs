use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CacheEntries::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CacheEntries::Namespace).text().not_null())
                    .col(ColumnDef::new(CacheEntries::Key).text().not_null())
                    .col(ColumnDef::new(CacheEntries::Value).binary().not_null())
                    .col(
                        ColumnDef::new(CacheEntries::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(Index::create().col(CacheEntries::Namespace).col(CacheEntries::Key))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_cache_entries_expires_at")
                    .table(CacheEntries::Table)
                    .col(CacheEntries::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CacheEntries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CacheEntries {
    Table,
    Namespace,
    Key,
    Value,
    ExpiresAt,
}
