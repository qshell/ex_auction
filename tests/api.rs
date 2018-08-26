#[macro_use]
extern crate assert_matches;
extern crate exonum;
extern crate ex_auction;
extern crate exonum_testkit;
#[macro_use]
extern crate serde_json;

use exonum::{
    api::{self, node::public::explorer::TransactionQuery},
    crypto::{self, CryptoHash, Hash, PublicKey, SecretKey},
};
use exonum_testkit::{ApiKind, TestKit, TestKitApi, TestKitBuilder};

use ex_auction::api::{LotQuery, BidQuery, LotBidsQuery};
use ex_auction::schema::{Lot, Bid};
use ex_auction::AuctionService;
use ex_auction::transactions::{TxCreateLot, TxCreateBid, TxCloseLot};
use ex_auction::AUCTION_SERVICE_NAME;


#[test]
fn test_create_lot() {
    let (mut testkit, api) = create_testkit();
    let token_hash = crypto::hash(&[0]);
    let description = "Lot 1";
    let (tx, _, _, _) = api.create_lot(&token_hash, description, 100);
    testkit.create_block();
    api.assert_tx_status(tx.hash(), &json!({ "type": "success" }));

    let lot = api.get_lot(*tx.pub_key());
    assert_eq!(lot.pub_key(), tx.pub_key());
    assert_eq!(lot.token_hash(), tx.token_hash());
    assert_eq!(lot.description(), tx.description());
    assert_eq!(lot.price(), 100);
}


#[test]
fn test_create_bid() {
    let (mut testkit, api) = create_testkit();
    let token_hash = crypto::hash(&[0]);
    let description = "Lot 2";
    let (lot_tx, _, _, _) = api.create_lot(&token_hash, description, 100);
    testkit.create_block();

    api.assert_tx_status(lot_tx.hash(), &json!({ "type": "success" }));
    let lot = api.get_lot(*lot_tx.pub_key());
    let (bid_tx, _) = api.create_bid(lot.pub_key(),lot.pub_session_key(), 101, 1);
    testkit.create_block();

    api.assert_tx_status(bid_tx.hash(), &json!({ "type": "success" }));
    let bid = api.get_bid(*bid_tx.pub_key());
    assert_eq!(bid.pub_key(), bid_tx.pub_key());
    assert_eq!(bid.data(), bid_tx.data());
}


#[test]
fn test_bids_count() {
    let (mut testkit, api) = create_testkit();
    let token_hash = crypto::hash(&[0]);
    let description = "Lot 3";
    let (lot_tx, _, _, _) = api.create_lot(&token_hash, description, 100);
    testkit.create_block();

    api.assert_tx_status(lot_tx.hash(), &json!({ "type": "success" }));
    let lot = api.get_lot(*lot_tx.pub_key());
    let (bid_tx1, _) = api.create_bid(lot.pub_key(), lot.pub_session_key(), 101, 1);
    testkit.create_block();
    let (bid_tx2, _) = api.create_bid(lot.pub_key(), lot.pub_session_key(), 102, 2);
    testkit.create_block();

    let bids = api.get_lot_bids(*lot.pub_key());
    assert_eq!(bids.len(), 2);
}


#[test]
fn test_close_lot1() {
    let (mut testkit, api) = create_testkit();
    let token_hash = crypto::hash(&[0]);
    let description = "Lot 4";
    let (lot_tx, pub_session_key, secret_session_key, secret_key) = api.create_lot(&token_hash, description, 100);
    testkit.create_block();

    api.assert_tx_status(lot_tx.hash(), &json!({ "type": "success" }));
    let lot = api.get_lot(*lot_tx.pub_key());
    let (bid_tx1, _) = api.create_bid(lot.pub_key(), lot.pub_session_key(), 101, 1);
    testkit.create_block();
    let (bid_tx2, _) = api.create_bid(lot.pub_key(), lot.pub_session_key(), 102, 2);
    testkit.create_block();

    let close_lot_tx = api.close_lot(lot.pub_key(), &pub_session_key, &secret_session_key, &secret_key);
    testkit.create_block();
    api.assert_tx_status(close_lot_tx.hash(), &json!({ "type": "success" }));
    let lot = api.get_lot(*lot_tx.pub_key());
    println!("{:?}", lot);
}


fn create_testkit() -> (TestKit, AuctionApi) {
    let (pub_session_key, secret_session_key) = exonum::crypto::gen_keypair();
    let testkit = TestKitBuilder::validator()
        .with_service(AuctionService)
        .create();
    let api = AuctionApi {
        inner: testkit.api(),
    };
    (testkit, api)
}


struct AuctionApi {
    pub inner: TestKitApi
}


impl AuctionApi {
    fn create_lot(&self, token_hash: &Hash, description: &str, price: u64) -> (TxCreateLot, PublicKey, SecretKey, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();
        let (pub_session_key, secret_session_key) = crypto::gen_keypair();
        let tx = TxCreateLot::new(&pubkey, &pub_session_key, token_hash, description, price, &key);

        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service(AUCTION_SERVICE_NAME))
            .query(&tx)
            .post("lot")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, pub_session_key, secret_session_key, key)
    }

    fn close_lot(&self, lot_pub_key: &PublicKey, pub_session_key: &PublicKey, secret_session_key: &SecretKey, secret_key: &SecretKey) -> TxCloseLot {
        let tx = TxCloseLot::new(&lot_pub_key, &pub_session_key, &secret_session_key.to_hex(), &secret_key);
        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service(AUCTION_SERVICE_NAME))
            .query(&tx)
            .post("close_lot")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        tx
    }

    fn create_bid(&self, lot_pub_key: &PublicKey, pub_session_key: &PublicKey, sum: u64, member_id: u64) -> (TxCreateBid, SecretKey) {
        let (pubkey, key) = crypto::gen_keypair();

        let bid = Bid::new(&pubkey, lot_pub_key, member_id, sum, false, vec![]); //.encrypt(pub_session_key);
        let tx = TxCreateBid::new(&pubkey, lot_pub_key, bid.data(), &key);

        let tx_info: serde_json::Value = self.inner
            .public(ApiKind::Service(AUCTION_SERVICE_NAME))
            .query(&tx)
            .post("bid")
            .unwrap();
        assert_eq!(tx_info, json!({ "tx_hash": tx.hash() }));
        (tx, key)
    }

    fn assert_tx_status(&self, tx_hash: Hash, expected_status: &serde_json::Value) {
        let info: serde_json::Value = self.inner
            .public(ApiKind::Explorer)
            .query(&TransactionQuery::new(tx_hash))
            .get("v1/transactions")
            .unwrap();

        if let serde_json::Value::Object(mut info) = info {
            let tx_status = info.remove("status").unwrap();
            assert_eq!(tx_status, *expected_status);
        } else {
            panic!("Invalid transaction info format, object expected");
        }
    }

    fn get_lot(&self, pub_key: PublicKey) -> Lot {
        self.inner
            .public(ApiKind::Service(AUCTION_SERVICE_NAME))
            .query(&LotQuery { pub_key })
            .get("lot")
            .unwrap()
    }

    fn get_bid(&self, pub_key: PublicKey) -> Bid {
        self.inner
            .public(ApiKind::Service(AUCTION_SERVICE_NAME))
            .query(&BidQuery { pub_key })
            .get("bid")
            .unwrap()
    }

    fn get_lot_bids(&self, lot_pub_key: PublicKey) -> Vec<Bid> {
        self.inner
            .public(ApiKind::Service(AUCTION_SERVICE_NAME))
            .query(&LotBidsQuery { lot_pub_key })
            .get("lot_bids")
            .unwrap()
    }
}