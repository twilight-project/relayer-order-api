pub mod config;
pub mod kafkalib;
pub mod relayer;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod test {

    use relayerarchiverlib::{database::TraderOrder as TraderOrderDB, RelayerDB};
    #[test]
    fn test_check_get_order_by_uuid_from_archiver() {
        dotenv::dotenv().expect("Failed loading dotenv");
        let database_url = std::env::var("DATABASE_URL").expect("No database url found!");
        let relayer_db = RelayerDB::from_host(database_url);
        let mut pool = relayer_db.get_conn().unwrap();
        TraderOrderDB::get_by_uuid(&mut *pool, "".to_string());
    }
}
