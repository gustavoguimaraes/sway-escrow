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
    let wallets =
        launch_custom_provider_and_get_wallets(WalletsConfig::default(), None, None).await;

    let deployer: &WalletUnlocked = wallets.get(0).unwrap();
    let creator: &WalletUnlocked = &wallets[1];
    let receiver: &WalletUnlocked = &wallets[2];

    let rng: &mut StdRng = &mut StdRng::seed_from_u64(2322u64);
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
        wallet: deployer.clone(),
    };

    let creator = WalletAndInstance {
        escrow: Escrow::new(escrow_contract_id.clone(), creator.clone()),
        wallet: creator.clone(),
    };

    let receiver = WalletAndInstance {
        escrow: Escrow::new(escrow_contract_id.clone(), receiver.clone()),
        wallet: receiver.clone(),
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

#[tokio::test]
async fn can_initialize_escrow() {
    let (_, creator, receiver, _, creator_asset_id, receiver_asset_id) = setup_tests().await;

    let creator_asset_amount = 1_000;
    let requested_amount = 1000;

    let tx_params: TxParameters = TxParameters::new(None, Some(1_000_000), None);

    let call_params = CallParameters::new(
        Some(creator_asset_amount),
        Some(AssetId::from(*creator_asset_id)),
        None,
    );

    let receiver_contract_id = ContractId::new(receiver_asset_id.into());

    let result = creator
        .escrow
        .methods()
        .create(
            Identity::Address(receiver.wallet.address().into()),
            receiver_contract_id,
            requested_amount,
        )
        .tx_params(tx_params)
        .call_params(call_params)
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, 0);
}
