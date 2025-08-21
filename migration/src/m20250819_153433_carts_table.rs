use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Carts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Carts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(Carts::UserId)) // still string
                    .col(
                        ColumnDef::new(Carts::ProductId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Carts::TotalQty)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Carts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .col(
                        ColumnDef::new(Carts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Carts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Carts {
    Table,
    Id,
    UserId,
    ProductId,
    TotalQty,
    CreatedAt,
    UpdatedAt,
}
