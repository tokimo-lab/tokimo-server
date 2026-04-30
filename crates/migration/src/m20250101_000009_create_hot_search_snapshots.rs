use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HotSearchSnapshots::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HotSearchSnapshots::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(ColumnDef::new(HotSearchSnapshots::Source).text().not_null())
                    .col(
                        ColumnDef::new(HotSearchSnapshots::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT now()"),
                    )
                    .col(ColumnDef::new(HotSearchSnapshots::Payload).json_binary().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_hot_search_snapshots_source")
                    .table(HotSearchSnapshots::Table)
                    .col(HotSearchSnapshots::Source)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_hot_search_snapshots_fetched_at")
                    .table(HotSearchSnapshots::Table)
                    .col(HotSearchSnapshots::FetchedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HotSearchSnapshots::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HotSearchSnapshots {
    Table,
    Id,
    Source,
    FetchedAt,
    Payload,
}
