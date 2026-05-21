use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX idx_hot_search_snapshots_source;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX idx_hot_search_snapshots_fetched_at;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_hot_search_snapshots_source_fetched_at ON hot_search_snapshots (source, fetched_at DESC);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP INDEX idx_hot_search_snapshots_source_fetched_at;")
            .await?;
        manager
            .get_connection()
            .execute_unprepared("CREATE INDEX idx_hot_search_snapshots_source ON hot_search_snapshots (source);")
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX idx_hot_search_snapshots_fetched_at ON hot_search_snapshots (fetched_at);",
            )
            .await?;

        Ok(())
    }
}
