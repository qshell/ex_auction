extern crate exonum_sodiumoxide;

use std::mem;
use exonum::{
    crypto::{PublicKey, Hash, SecretKey}, storage::{Fork, MapIndex, ListIndex, Snapshot},
    helpers
};
use serde_json;
use schema::exonum_sodiumoxide::crypto;
use utils::{hex_to_bytes, from_slice};


encoding_struct! {
    struct Lot {
        pub_key: &PublicKey,
        pub_session_key: &PublicKey,
        token_hash: &Hash,
        description: &str,
        price: u64,
        opened: bool,
        winner_member_id: u64
    }
}


impl Lot {
    pub fn close(self, winner_member_id: u64, price: u64) -> Self {
        Self::new(self.pub_key(), self.pub_session_key(), self.token_hash(), self.description(), price, false, winner_member_id)
    }
}


encoding_struct! {
    struct Bid {
        pub_key: &PublicKey,
        lot_pub_key: &PublicKey,
        member_id: u64,
        sum: u64,
        encrypted: bool,
        data: Vec<u8>
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BidData {
    member_id: u64,
    sum: u64
}


impl Bid {
    pub fn decrypt(self, pub_session_key: &PublicKey, secret_session_key_hex: &str) -> Bid {
        let pub_session_key_bytes = hex_to_bytes(&pub_session_key.to_hex());
        let secret_session_key_bytes = hex_to_bytes(secret_session_key_hex);
        let pub_session_key = crypto::box_::PublicKey(from_slice(&pub_session_key_bytes));
        let secret_session_key = crypto::box_::SecretKey(from_slice(&secret_session_key_bytes));
        let data = crypto::sealedbox::open(
            &self.data(),
            &pub_session_key,
            &secret_session_key
        );
        match data {
            Err(e) => {
                println!("{:?}", e);
                self
            },
            Ok(v) => {
                let data: BidData = serde_json::from_slice(&v).unwrap();
                Self::new(self.pub_key(), self.lot_pub_key(), data.member_id, data.sum, false, vec![])
            }
        }
    }

    pub fn encrypt(self, pub_session_key: &PublicKey) -> Bid {
        let pub_session_key_bytes = hex_to_bytes(&pub_session_key.to_hex());
        let pub_session_key = crypto::box_::PublicKey(from_slice(&pub_session_key_bytes));
        let data = BidData {member_id: self.member_id(), sum: self.sum()};
        let data = serde_json::to_vec(&data).unwrap();
        let data = crypto::sealedbox::seal(&data, &pub_session_key);
        Self::new(self.pub_key(), self.lot_pub_key(), 0, 0, true, data)
        /*unsafe {
            let pub_session_key = mem::transmute::<PublicKey, crypto::box_::PublicKey>(*pub_session_key);
            //let sum: [u8; 8] = mem::transmute(self.sum().to_be());
            //let member_id: [u8; 8] = mem::transmute(self.member_id().to_be());
            //let data = [sum, member_id].concat();
            let data = crypto::sealedbox::seal(&data, &pub_session_key);
            Self::new(self.pub_key(), self.lot_pub_key(), 0, 0, true, data)
        }*/
    }
}


#[derive(Debug)]
pub struct AuctionSchema<T> {
    view: T
}


impl<T: AsRef<dyn Snapshot>> AuctionSchema<T> {
    pub fn new(view: T) -> Self {
        AuctionSchema { view }
    }

    pub fn lots(&self) -> MapIndex<&dyn Snapshot, PublicKey, Lot> {
        MapIndex::new("ex_auction.lots", self.view.as_ref())
    }

    pub fn lot(&self, pub_key: &PublicKey) -> Option<Lot> {
        self.lots().get(pub_key)
    }

    pub fn lot_bids(&self, lot_pub_key: &PublicKey) -> ListIndex<&dyn Snapshot, PublicKey> {
        ListIndex::new_in_family("ex_auction.lot_bids", lot_pub_key, self.view.as_ref())
    }

    pub fn bids(&self) -> MapIndex<&dyn Snapshot, PublicKey, Bid> {
        MapIndex::new("ex_auction.bids", self.view.as_ref())
    }

    pub fn bid(&self, pub_key: &PublicKey) -> Option<Bid> {
        self.bids().get(pub_key)
    }
}


impl<'a> AuctionSchema<&'a mut Fork> {
    pub fn lots_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Lot> {
        MapIndex::new("ex_auction.lots", &mut self.view)
    }

    pub fn bids_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Bid> {
        MapIndex::new("ex_auction.bids", &mut self.view)
    }

    pub fn lot_bids_mut(&mut self, lot_public_key: &PublicKey) -> ListIndex<&mut Fork, PublicKey> {
        ListIndex::new_in_family("ex_auction.lot_bids", lot_public_key, &mut self.view)
    }
}
