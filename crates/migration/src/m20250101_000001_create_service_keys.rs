use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceKeys::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ServiceKeys::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(ColumnDef::new(ServiceKeys::Name).text().not_null())
                    .col(ColumnDef::new(ServiceKeys::TokenHash).text().not_null())
                    .col(ColumnDef::new(ServiceKeys::TokenPrefix).text().not_null())
                    .col(
                        ColumnDef::new(ServiceKeys::Scopes)
                            .json_binary()
                            .not_null()
                            .default("'[]'"),
                    )
                    .col(ColumnDef::new(ServiceKeys::Enabled).boolean().not_null().default(true))
                    .col(ColumnDef::new(ServiceKeys::ExpiresAt).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(ServiceKeys::CreatedAt)
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
                    .name("idx_service_keys_token_prefix")
                    .table(ServiceKeys::Table)
                    .col(ServiceKeys::TokenPrefix)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ServiceKeys::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceKeys {
    Table,
    Id,
    Name,
    TokenHash,
    TokenPrefix,
    Scopes,
    Enabled,
    ExpiresAt,
    CreatedAt,
}
