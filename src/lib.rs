#[macro_use]
extern crate exonum;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate failure;

use exonum::{
    api::ServiceApiBuilder, blockchain::{Service, Transaction, TransactionSet},
    crypto::Hash,
    encoding, messages::RawTransaction, storage::Snapshot
};

pub mod api;
pub mod schema;
pub mod transactions;
pub mod errors;
pub mod utils;

use api::AuctionApi;
use transactions::AuctionTransactions;


pub const AUCTION_SERVICE_ID: u16 = 17;
pub const AUCTION_SERVICE_NAME: &str = "ex_auction";



#[derive(Debug)]
pub struct AuctionService;


impl Service for AuctionService {
    fn service_id(&self) -> u16 {
        AUCTION_SERVICE_ID
    }

    fn service_name(&self) -> &'static str {
        AUCTION_SERVICE_NAME
    }

    fn state_hash(&self, _: &dyn Snapshot) -> Vec<Hash> {
        vec![]
    }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<dyn Transaction>, encoding::Error> {
        let tx = AuctionTransactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        AuctionApi::wire(builder);
    }
}
