pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m20250325_204952_init;
pub(crate) mod table_types;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250325_204952_init::Migration)]
    }
}
