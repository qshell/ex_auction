use exonum::{
    blockchain::{ExecutionError, ExecutionResult, Transaction}, crypto::{Hash, PublicKey, SecretKey},
    messages::Message, storage::Fork,
};
use schema::{AuctionSchema, Lot, Bid};
use errors::Error;

use AUCTION_SERVICE_ID;


transactions! {
    pub AuctionTransactions {
        const SERVICE_ID = AUCTION_SERVICE_ID;

        struct TxCreateLot {
            pub_key: &PublicKey,
            pub_session_key: &PublicKey,
            token_hash: &Hash,
            description: &str,
            price: u64
        }

        struct TxCreateBid {
            pub_key: &PublicKey,
            lot_pub_key: &PublicKey,
            data: Vec<u8>
        }

        struct TxCloseLot {
            lot_pub_key: &PublicKey,
            pub_session_key: &PublicKey,
            secret_session_key: &str
        }
    }
}


impl Transaction for TxCreateLot {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(view);
        if schema.lot(self.pub_key()).is_none() {
            let lot = Lot::new(self.pub_key(), self.pub_session_key(), self.token_hash(), self.description(), self.price(), true, 0);
            println!("Create the lot: {:?}", lot);
            schema.lots_mut().put(self.pub_key(), lot);
            Ok(())
        } else {
            Err(Error::LotAlreadyExists)?
        }
    }
}


impl Transaction for TxCreateBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(view);
        if schema.lot(self.lot_pub_key()).is_none() {
            Err(Error::LotNotFound)?
        }
        let bid = Bid::new(self.pub_key(), self.lot_pub_key(), 0, 0, true, self.data());
        println!("Create the bid: {:?}", bid);
        schema.lot_bids_mut(self.lot_pub_key()).push(*bid.pub_key());
        schema.bids_mut().put(self.pub_key(), bid);
        Ok(())
    }
}


impl Transaction for TxCloseLot {
    fn verify(&self) -> bool {
        self.verify_signature(self.lot_pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = AuctionSchema::new(view);
        let lot = schema.lot(self.lot_pub_key());
        if lot.is_none() {
            Err(Error::LotNotFound)?
        };
        let lot = lot.unwrap();
        let bid_pub_keys: Vec<PublicKey>;
        {
            let lot_bids = schema.lot_bids_mut(self.lot_pub_key());
            bid_pub_keys = lot_bids.iter().collect();
        }
        let mut winner_member_id: u64 = 0;
        let mut winner_sum: u64 = 0;
        for bid_pub_key in bid_pub_keys.iter() {
            let bid = schema.bids_mut().get(&bid_pub_key).unwrap();
            if bid.sum() > winner_sum {
                winner_member_id = bid.member_id();
                winner_sum = bid.sum();
            }
            // TODO: fix decrypt
            //let bid = bid.decrypt(self.pub_session_key(), self.secret_session_key());

            schema.bids_mut().put(&bid_pub_key, bid);
        }
        let lot = lot.close(winner_member_id, winner_sum);
        schema.lots_mut().put(self.lot_pub_key(), lot);

        Ok(())
    }
}
