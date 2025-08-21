pub use sea_orm_migration::prelude::*;

mod m20250804_005445_categories_table;
mod m20250811_011544_products_table;
mod m20250811_024226_add_product_img_url_in_products_table;
mod m20250819_153433_carts_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250804_005445_categories_table::Migration),
            Box::new(m20250811_011544_products_table::Migration),
            Box::new(m20250811_024226_add_product_img_url_in_products_table::Migration),
            Box::new(m20250819_153433_carts_table::Migration),
        ]
    }
}
