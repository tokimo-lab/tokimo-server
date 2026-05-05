use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProviderConfigs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProviderConfigs::Provider)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProviderConfigs::Config)
                            .json_binary()
                            .not_null()
                            .extra("DEFAULT '{}'::jsonb"),
                    )
                    .col(
                        ColumnDef::new(ProviderConfigs::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(ProviderConfigs::UpdatedAt)
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
            .drop_table(Table::drop().table(ProviderConfigs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ProviderConfigs {
    Table,
    Provider,
    Config,
    Enabled,
    UpdatedAt,
}
