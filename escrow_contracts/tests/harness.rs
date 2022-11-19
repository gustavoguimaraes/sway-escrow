use fuels::prelude::*;
use fuels::signers::wallet::WalletUnlocked;
use fuels::tx::ContractId;
use fuels_abigen_macro::abigen;
use rand::prelude::{Rng, SeedableRng, StdRng};

#[allow(dead_code)]
struct WalletAndInstance {
    escrow: Escrow,
    wallet: WalletUnlocked,
}

// load abi from json
abigen!(Escrow, "out/debug/escrow-abi.json");
abigen!(Asset, "../asset/out/debug/asset-abi.json");

async fn setup_tests() -> (
    WalletAndInstance,
    WalletAndInstance,
    WalletAndInstance,
    ContractId,
    ContractId,
    ContractId,
) {
    // let num_wallets = 3;
    // let coins_per_wallet = 1;
    // let amount_per_coin = 1_000_000;

    let deployer = WalletUnlocked::new_random(None);
    let creator = WalletUnlocked::new_random(None);
    let receiver = WalletUnlocked::new_random(None);

    let rng = &mut StdRng::seed_from_u64(2322u64);
    let salt: [u8; 32] = rng.gen();

    let creator_asset_id = Contract::deploy_with_parameters(
        "../asset/out/debug/asset.bin",
        &deployer,
        TxParameters::default(),
        StorageConfiguration::default(),
        Salt::from(salt),
    )
    .await
    .unwrap();

    let salt: [u8; 32] = rng.gen();
    let receiver_asset_id = Contract::deploy_with_parameters(
        "../asset/out/debug/asset.bin",
        &deployer,
        TxParameters::default(),
        StorageConfiguration::default(),
        Salt::from(salt),
    )
    .await
    .unwrap();

    let creator_asset_instance = Asset::new(creator_asset_id.clone(), deployer.clone());

    let receiver_asset_instance = Asset::new(receiver_asset_id.clone(), deployer.clone());

    creator_asset_instance
        .methods()
        .mint_and_send_to_address(1_000_000, Identity::Address(creator.address().into()))
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap()
        .value;
    receiver_asset_instance
        .methods()
        .mint_and_send_to_address(1_000_000, Identity::Address(receiver.address().into()))
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap()
        .value;

    let escrow_contract_id = Contract::deploy(
        "./out/debug/escrow.bin",
        &deployer,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let deployer = WalletAndInstance {
        escrow: Escrow::new(escrow_contract_id.clone(), deployer.clone()),
        wallet: deployer,
    };

    let creator = WalletAndInstance {
        escrow: Escrow::new(escrow_contract_id.clone(), creator.clone()),
        wallet: creator,
    };

    let receiver = WalletAndInstance {
        escrow: Escrow::new(escrow_contract_id.clone(), receiver.clone()),
        wallet: receiver,
    };
    println!("asset ids {:?} {:?}", creator_asset_id, receiver_asset_id);

    return (
        deployer,
        creator,
        receiver,
        escrow_contract_id.into(),
        creator_asset_id.into(),
        receiver_asset_id.into(),
    );
}
