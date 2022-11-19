library escrow;

use std::{contract_id::ContractId, identity::Identity};

abi escrow {
    #[storage(read, write)]
    fn create(receiver: Identity, requested_asset_id: ContractId, requested_asset_amount: u64) -> u64;
    #[storage(read, write)]
    fn accept(escrow_id: u64);
    #[storage(read, write)]
    fn revert(escrow_id: u64);
}

pub enum Status {
    Uninitialized: (),
    Completed: (),
    Reverted: (),
}

pub struct EscrowInstance {
    creator: Identity,
    receiver: Identity,
    creator_asset_id: ContractId,
    creator_asset_amount: u64,
    requested_asset_id: ContractId,
    requested_asset_amount: u64,
    status: Status,
}
