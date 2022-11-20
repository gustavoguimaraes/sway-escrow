contract;

dep escrow;
use escrow::*;

use std::{
    auth::{
        AuthError,
        msg_sender,
    },
    call_frames::msg_asset_id,
    context::msg_amount,
    contract_id::ContractId,
    identity::Identity,
    storage::StorageMap,
    token::transfer_to_address,
};

storage {
    escrows: StorageMap<u64, EscrowInstance> = StorageMap {},
    escrow_index: u64 = 0,
}

pub enum EscrowError {
    IncorrectEscrowState: (),
    IncorrectAssetReceived: (),
    IncorrectReceiver: (),
    InsufficientAmountReceived: (),
}

impl escrow for Contract {
    #[storage(read, write)]
    fn create(
        receiver: Identity,
        requested_asset_id: ContractId,
        requested_asset_amount: u64,
    ) -> u64 {
        let escrow_id = storage.escrow_index;

        let escrow_instance = EscrowInstance {
            creator: msg_sender().unwrap(),
            creator_asset_id: msg_asset_id(),
            creator_asset_amount: msg_amount(),
            receiver: receiver,
            requested_asset_id: requested_asset_id,
            requested_asset_amount: requested_asset_amount,
            status: Status::Uninitialized,
        };

        storage.escrows.insert(escrow_id, escrow_instance);
        storage.escrow_index += 1;

        return escrow_id;
    }

    #[storage(read, write)]
    fn accept(escrow_id: u64) {
        let mut escrow_instance = storage.escrows.get(escrow_id);

        require(escrow_instance.status == Status::Uninitialized, EscrowError::IncorrectEscrowState);

        require(escrow_instance.requested_asset_id == msg_asset_id(), EscrowError::IncorrectAssetReceived);

        require(escrow_instance.requested_asset_amount <= msg_amount(), EscrowError::InsufficientAmountReceived);

        require(escrow_instance.receiver == msg_sender().unwrap(), EscrowError::IncorrectReceiver);

        escrow_instance.status = Status::Completed;

        storage.escrows.insert(escrow_id, escrow_instance);

        let creator_address = match escrow_instance.creator {
            Identity::Address(address) => address,
            _ => revert(0),
        };
        transfer_to_address(escrow_instance.requested_asset_amount, escrow_instance.requested_asset_id, creator_address);

        let receiver_address = match escrow_instance.receiver {
            Identity::Address(address) => address,
            _ => revert(0),
        };

        // transfer_to_address means transfer amount of coin of given type `asset_id` and send them to the address `to`
        transfer_to_address(escrow_instance.creator_asset_amount, escrow_instance.creator_asset_id, receiver_address);
    }

    #[storage(read, write)]
    fn revert(escrow_id: u64) {
        let mut escrow_instance = storage.escrows.get(escrow_id);

        require(escrow_instance.status == Status::Uninitialized, EscrowError::IncorrectEscrowState);

        require(escrow_instance.creator == msg_sender().unwrap(), EscrowError::IncorrectReceiver);
        escrow_instance.status = Status::Reverted;
        storage.escrows.insert(escrow_id, escrow_instance);

        let creator_address = match escrow_instance.creator {
            Identity::Address(address) => address,
            _ => revert(0),
        };

        transfer_to_address(escrow_instance.creator_asset_amount, escrow_instance.creator_asset_id, creator_address);
    }
}
