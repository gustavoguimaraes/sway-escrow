contract;

use std::{identity::Identity, token::mint_to};

abi Asset {
    #[storage(read, write)]
    fn mint_and_send_to_address(amount: u64, recipient: Identity) -> bool;
}

impl Asset for Contract {
    #[storage(read, write)]
    fn mint_and_send_to_address(amount: u64, recipient: Identity) -> bool {
        mint_to(amount, recipient);
        return true
    }
}
