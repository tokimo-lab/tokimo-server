use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(HotSearchItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(HotSearchItems::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(ColumnDef::new(HotSearchItems::Source).text().not_null())
                    .col(ColumnDef::new(HotSearchItems::IndexPos).integer().not_null())
                    .col(ColumnDef::new(HotSearchItems::Title).text().not_null())
                    .col(ColumnDef::new(HotSearchItems::Link).text().not_null())
                    .col(ColumnDef::new(HotSearchItems::HotValue).text())
                    .col(ColumnDef::new(HotSearchItems::Label).text())
                    .col(
                        ColumnDef::new(HotSearchItems::FetchedAt)
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
                    .name("idx_hot_search_items_unique")
                    .table(HotSearchItems::Table)
                    .col(HotSearchItems::Source)
                    .col(HotSearchItems::IndexPos)
                    .col(HotSearchItems::FetchedAt)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_hot_search_items_source")
                    .table(HotSearchItems::Table)
                    .col(HotSearchItems::Source)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(HotSearchItems::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum HotSearchItems {
    Table,
    Id,
    Source,
    IndexPos,
    Title,
    Link,
    HotValue,
    Label,
    FetchedAt,
}
