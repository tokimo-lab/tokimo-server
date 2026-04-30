use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SportMatches::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SportMatches::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(ColumnDef::new(SportMatches::MatchDate).date().not_null())
                    .col(ColumnDef::new(SportMatches::MatchType).text().not_null())
                    .col(ColumnDef::new(SportMatches::MatchName).text().not_null())
                    .col(ColumnDef::new(SportMatches::StartTime).text())
                    .col(ColumnDef::new(SportMatches::Status).text())
                    .col(ColumnDef::new(SportMatches::VsLine).text())
                    .col(ColumnDef::new(SportMatches::LeftTeam).json_binary())
                    .col(ColumnDef::new(SportMatches::RightTeam).json_binary())
                    .col(ColumnDef::new(SportMatches::Game).text())
                    .col(ColumnDef::new(SportMatches::Link).text())
                    .col(
                        ColumnDef::new(SportMatches::HasLive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SportMatches::FetchedAt)
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
                    .name("idx_sport_matches_match_date")
                    .table(SportMatches::Table)
                    .col(SportMatches::MatchDate)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SportMatches::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SportMatches {
    Table,
    Id,
    MatchDate,
    MatchType,
    MatchName,
    StartTime,
    Status,
    VsLine,
    LeftTeam,
    RightTeam,
    Game,
    Link,
    HasLive,
    FetchedAt,
}
