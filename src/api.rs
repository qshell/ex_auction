use exonum::api::{self, ServiceApiBuilder, ServiceApiState};
use exonum::crypto::{Hash, PublicKey};
use exonum::blockchain::Transaction;
use exonum::node::TransactionSend;
use schema as ex_schema;
use transactions::AuctionTransactions;


#[derive(Debug, Clone, Copy)]
pub struct AuctionApi;


#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct LotQuery {
    pub pub_key: PublicKey
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BidQuery {
    pub pub_key: PublicKey
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct LotBidsQuery {
    pub lot_pub_key: PublicKey
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx_hash: Hash
}


impl AuctionApi {
    pub fn get_lot(state: &ServiceApiState, query: LotQuery) -> api::Result<ex_schema::Lot> {
        let snapshot = state.snapshot();
        let schema = ex_schema::AuctionSchema::new(snapshot);
        schema.lot(&query.pub_key)
            .ok_or_else(|| api::Error::NotFound("\"Lot is not found\"".to_owned()))
    }

    pub fn get_bid(state: &ServiceApiState, query: BidQuery) -> api::Result<ex_schema::Bid> {
        let snapshot = state.snapshot();
        let schema = ex_schema::AuctionSchema::new(snapshot);
        schema.bid(&query.pub_key)
            .ok_or_else(|| api::Error::NotFound("\"Bid is not found\"".to_owned()))
    }

    pub fn get_lot_bids(state: &ServiceApiState, query: LotBidsQuery) -> api::Result<Vec<ex_schema::Bid>> {
        let snapshot = state.snapshot();
        let schema = ex_schema::AuctionSchema::new(snapshot);
        let lot = schema.lot(&query.lot_pub_key)
            .ok_or_else(|| api::Error::NotFound("\"Lot is not found\"".to_owned()));
        if let Err(e) = lot {return Err(e)}
        let lot_bids = schema.lot_bids(&query.lot_pub_key);
        let bids: Vec<ex_schema::Bid> = lot_bids.iter().map(|pub_key| schema.bid(&pub_key).unwrap()).collect();
        Ok(bids)
    }

    pub fn post_transaction(state: &ServiceApiState, query: AuctionTransactions) -> api::Result<TransactionResponse> {
        let transaction: Box<dyn Transaction> = query.into();
        let tx_hash = transaction.hash();
        state.sender().send(transaction)?;
        Ok(TransactionResponse { tx_hash })
    }

    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
        .public_scope()
        .endpoint("lot", Self::get_lot)
        .endpoint_mut("lot", Self::post_transaction)
        .endpoint("bid", Self::get_bid)
        .endpoint_mut("bid", Self::post_transaction)
        .endpoint("lot_bids", Self::get_lot_bids)
        .endpoint_mut("close_lot", Self::post_transaction);
    }
}
