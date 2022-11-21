use fuels::prelude::*;
use fuels::signers::wallet::WalletUnlocked;
use fuels::tx::{AssetId, ContractId};
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

async fn get_contract_instance(wallet: WalletUnlocked) -> (Escrow, ContractId) {
    // Launch a local network and deploy the contract

    let escrow_contract_id = Contract::deploy(
        "./out/debug/escrow.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();

    let instance = Escrow::new(escrow_contract_id.clone(), wallet.clone());

    return (instance, escrow_contract_id.into());
}

#[tokio::test]
async fn can_initialize_contract() {
    let wallet = launch_provider_and_get_wallet().await;
    let (_, _) = get_contract_instance(wallet).await;
}

#[tokio::test]
async fn can_initialize_escrow() {
    let (_, creator, receiver, _, creator_asset_id, receiver_asset_id) = setup_tests().await;

    let creator_asset_amount = 100;
    let requested_amount = 100;

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

#[tokio::test]
async fn can_accept_escrow() {
    let (_, creator, receiver, _, creator_asset_id, receiver_asset_id) = setup_tests().await;

    let creator_asset_amount = 100;
    let requested_amount = 100;

    let creator_asset_id = Some(AssetId::from(*creator_asset_id)).unwrap();
    let receiver_asset_id: AssetId = Some(AssetId::from(*receiver_asset_id)).unwrap();

    // Get asset balances
    let creator_initial_creator_balance = creator
        .wallet
        .get_asset_balance(&creator_asset_id)
        .await
        .unwrap();
    let creator_initial_receiver_balance = creator
        .wallet
        .get_asset_balance(&receiver_asset_id)
        .await
        .unwrap();

    let receiver_initial_creator_balance = receiver
        .wallet
        .get_asset_balance(&creator_asset_id)
        .await
        .unwrap();
    let receiver_initial_receiver_balance = receiver
        .wallet
        .get_asset_balance(&receiver_asset_id)
        .await
        .unwrap();
    // end get asset balances
    assert_eq!(creator_initial_receiver_balance, 0);
    assert_eq!(receiver_initial_creator_balance, 0);

    // creator creates escrow
    let create_tx_params: TxParameters = TxParameters::new(None, Some(1_000_000), None);
    let create_call_params =
        CallParameters::new(Some(creator_asset_amount), Some(creator_asset_id), None);

    let receiver_contract_id = ContractId::new(receiver_asset_id.into());

    let escrow_id = creator
        .escrow
        .methods()
        .create(
            Identity::Address(receiver.wallet.address().into()),
            receiver_contract_id,
            requested_amount,
        )
        .tx_params(create_tx_params)
        .call_params(create_call_params)
        .call()
        .await
        .unwrap();

    let creator_current_creator_balance = creator
        .wallet
        .get_asset_balance(&creator_asset_id)
        .await
        .unwrap();
    let creator_current_receiver_balance = creator
        .wallet
        .get_asset_balance(&receiver_asset_id)
        .await
        .unwrap();

    let receiver_current_creator_balance = receiver
        .wallet
        .get_asset_balance(&creator_asset_id)
        .await
        .unwrap();
    let receiver_current_receiver_balance = receiver
        .wallet
        .get_asset_balance(&receiver_asset_id)
        .await
        .unwrap();

    // assertions for creating escrow
    assert_eq!(escrow_id.value, 0);

    assert_eq!(
        creator_current_creator_balance,
        creator_initial_creator_balance
            .checked_sub(creator_asset_amount)
            .unwrap()
    );
    assert_eq!(
        creator_current_receiver_balance,
        creator_initial_receiver_balance
    );
    assert_eq!(
        receiver_current_receiver_balance,
        receiver_initial_receiver_balance
    );
    assert_eq!(
        receiver_current_creator_balance,
        receiver_initial_creator_balance
    );
    // end of create escrow with creator

    // receiver accepts escrow
    let receive_tx_params: TxParameters = TxParameters::new(None, Some(1_000_000), None);
    let receive_call_params =
        CallParameters::new(Some(requested_amount), Some(receiver_asset_id), None);

    receiver
        .escrow
        .methods()
        .accept(escrow_id.value)
        .append_variable_outputs(2)
        .tx_params(receive_tx_params)
        .call_params(receive_call_params)
        .call()
        .await
        .unwrap();

    let creator_current_creator_balance = creator
        .wallet
        .get_asset_balance(&creator_asset_id)
        .await
        .unwrap();
    let creator_current_receiver_balance = creator
        .wallet
        .get_asset_balance(&receiver_asset_id)
        .await
        .unwrap();

    let receiver_current_creator_balance = receiver
        .wallet
        .get_asset_balance(&creator_asset_id)
        .await
        .unwrap();
    let receiver_current_receiver_balance = receiver
        .wallet
        .get_asset_balance(&receiver_asset_id)
        .await
        .unwrap();
    // assertions for accepting escrow
    assert_eq!(
        creator_current_creator_balance,
        creator_initial_creator_balance
            .checked_sub(creator_asset_amount)
            .unwrap()
    );
    assert_eq!(
        creator_current_receiver_balance,
        creator_initial_receiver_balance
            .checked_add(requested_amount)
            .unwrap()
    );
    assert_eq!(
        receiver_current_receiver_balance,
        receiver_initial_receiver_balance
            .checked_sub(requested_amount)
            .unwrap()
    );
    assert_eq!(
        receiver_current_creator_balance,
        receiver_initial_creator_balance
            .checked_add(creator_asset_amount)
            .unwrap()
    );
    // end of receiver accepts escrow

    // Experiment: another assertion
    let asset_id: AssetId = AssetId::new(*receiver_contract_id);
    let creator_requested_funds_bal: Result<u64, Error> =
        creator.wallet.get_asset_balance(&asset_id).await;
    let result: u64 = creator_requested_funds_bal.unwrap();

    print!("asset_id {:?}, balance {:?}", asset_id, result);

    assert_eq!(result, requested_amount);
}
