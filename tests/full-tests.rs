use scrypto::prelude::DIVISIBILITY_NONE;
use scrypto::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;
use transaction::prelude::ManifestBuilder;

#[test]
fn setup_component_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    setup_component(&main_account, XRD, XRD, &mut test_runner);
}

#[test]
fn add_accounts_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );
    // let test_str2 = r##"{"accounts":[["account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma","756.94"]],"orders":[{ "pair_address": "DEXTR/XRD", "pair_rewards": [["1303","1153.12"],["1306","14089.93"]]}]}"##;
    let (test_str, _account_addresses) = build_accounts_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(String::from(test_str), vec!(lookup.bucket("dextr_bucket"))),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2.clone(), None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        account_balance
    );
}

#[test]
fn add_orders_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address) =
        setup_component(&main_account, XRD, dextr_admin_token, &mut test_runner);
    // let test_str2 = r##"{"accounts":[["account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma","756.94"]],"orders":[{ "pair_address": "DEXTR/XRD", "pair_rewards": [["1303","1153.12"],["1306","14089.93"]]}]}"##;
    let test_str = build_orders_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(String::from(test_str), vec!(lookup.bucket("xrd_bucket"))),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2.clone(), None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), XRD);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8839.54"),
        "Expected Account Balance of 8839.54, but found {:?}",
        account_balance
    );
}

#[test]
pub fn claim_accounts_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address) =
        setup_component(&main_account, XRD, dextr_admin_token, &mut test_runner);
    let (test_str, test_accounts) = build_accounts_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2, XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(
                    String::from(test_str.trim()),
                    vec!(lookup.bucket("xrd_bucket"))
                ),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );

    // let receipt = test_runner.execute_manifest_ignoring_fee(tx_manifest.build(), tx_signers);
    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let main_account_balance = test_runner.get_component_balance(main_account.2, XRD);
    println!(
        "Account balance for main account: {:?}",
        main_account_balance
    );
    assert!(
        main_account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        main_account_balance
    );
    for (account_address_string, _, _expected_account_balance) in test_accounts.clone() {
        let account_address =
            ComponentAddress::try_from_hex(&account_address_string).expect(&format!(
                "Could not convert account address string {} into account address.",
                account_address_string
            ));
        let account_balance = test_runner.get_component_balance(account_address, XRD);
        println!(
            "Account balance for account {}: {:?}",
            account_address_string, account_balance
        );
    }

    let (test_account_string, test_account_pubkey, test_account_balance) = test_accounts[1].clone();
    let test_account_address = GlobalAddress::try_from_hex(&test_account_string).expect(&format!(
        "Could not create account global address from string {}",
        test_account_string
    ));
    let order_proofs: Vec<ManifestProof> = vec![];
    let tx_manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "claim_rewards",
            manifest_args!(vec!(test_account_address), order_proofs),
        )
        .try_deposit_entire_worktop_or_abort(test_account_address, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&test_account_pubkey)],
    );

    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let test_account_address =
        ComponentAddress::try_from_hex(&test_account_string).expect(&format!(
            "Could not convert account address string {} into account address.",
            test_account_string
        ));
    let account_balance = test_runner.get_component_balance(test_account_address, XRD);
    assert!(
        account_balance == test_account_balance,
        "Expected Account Balance of {:?}, but found {:?}",
        test_account_balance,
        account_balance
    );
    for (account_address_string, _account_pubkey, _expected_account_balance) in
        test_accounts.clone()
    {
        let account_address =
            ComponentAddress::try_from_hex(&account_address_string).expect(&format!(
                "Could not convert account address string {} into account address.",
                account_address_string
            ));
        let account_balance = test_runner.get_component_balance(account_address, XRD);
        println!(
            "Account balance for account {}: {:?}",
            account_address_string, account_balance
        );
    }
}

#[test]
pub fn remove_accounts_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );
    let (test_str, account_addresses) = build_accounts_test_str(&mut test_runner);
    println!("Add account rewards string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(String::from(test_str), vec!(lookup.bucket("dextr_bucket"))),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2.clone(), None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        account_balance
    );
    let account_addresses_only = account_addresses
        .clone()
        .into_iter()
        .map(|(account_address, _, _)| account_address)
        .collect();
    let remove_str = build_remove_accounts_test_str(account_addresses_only);
    println!("Remove rewards string: {:?}", remove_str);
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .call_method(
            component_address,
            "remove_rewards",
            manifest_args!(String::from(remove_str)),
        )
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2.clone(), None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let main_account_balance =
        test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Main Account balance: {:?}", main_account_balance);
    assert!(
        main_account_balance == dec!("9700"),
        "Expected Main Account Balance of 9700, but found {:?}",
        main_account_balance
    );
    let account1_address_string = account_addresses[0].clone().0;
    let account1_address =
        ComponentAddress::try_from_hex(&account1_address_string).expect(&format!(
            "Could not convert account1 address string {} into account address.",
            account1_address_string
        ));
    let account1_balance = test_runner.get_component_balance(account1_address, XRD);
    println!(
        "Account1 balance for account {}: {:?}",
        account1_address_string, account1_balance
    );
    let account2_address_string = account_addresses[1].clone().0;
    let account2_address =
        ComponentAddress::try_from_hex(&account2_address_string).expect(&format!(
            "Could not convert account2 address string {} into account address.",
            account2_address_string
        ));
    let account2_balance = test_runner.get_component_balance(account2_address, XRD);
    println!(
        "Account2 balance for account {}: {:?}",
        account2_address_string, account2_balance
    );
}

#[test]
pub fn remove_accounts_rewards_overflow_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );
    let (test_str, account_addresses) = build_accounts_test_str(&mut test_runner);
    println!("Add account rewards string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(String::from(test_str), vec!(lookup.bucket("dextr_bucket"))),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2.clone(), None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Main Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Main Account Balance of 8840.54, but found {:?}",
        account_balance
    );

    let (test_account_string, test_account_pubkey, test_account_balance) =
        account_addresses[1].clone();
    let test_account_address = GlobalAddress::try_from_hex(&test_account_string).expect(&format!(
        "Could not create account global address from string {}",
        test_account_string
    ));
    let order_proofs: Vec<ManifestProof> = vec![];
    let tx_manifest = ManifestBuilder::new()
        .call_method(
            component_address,
            "claim_rewards",
            manifest_args!(vec!(test_account_address), order_proofs),
        )
        .try_deposit_entire_worktop_or_abort(test_account_address, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&test_account_pubkey)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let test_account_address =
        ComponentAddress::try_from_hex(&test_account_string).expect(&format!(
            "Could not convert account address string {} into account address.",
            test_account_string
        ));
    let account_balance = test_runner.get_component_balance(test_account_address, XRD);

    println!("Claimed Account Balance: {:?}", account_balance);
    assert!(
        account_balance == test_account_balance,
        "Expected Account Balance of {:?}, but found {:?}",
        test_account_balance,
        account_balance
    );

    let account_addresses_only = account_addresses
        .clone()
        .into_iter()
        .map(|(account_address, _, _)| account_address)
        .collect();
    let remove_str = build_remove_accounts_test_str(account_addresses_only);
    println!("Remove rewards string: {:?}", remove_str);
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .call_method(
            component_address,
            "remove_rewards",
            manifest_args!(String::from(remove_str)),
        )
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2.clone(), None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let main_account_balance =
        test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Main Account balance: {:?}", main_account_balance);
    assert!(
        main_account_balance == dec!("9098.33"),
        "Expected Main Account Balance of 9098.33, but found {:?}",
        main_account_balance
    );
    let account1_address_string = account_addresses[0].clone().0;
    let account1_address =
        ComponentAddress::try_from_hex(&account1_address_string).expect(&format!(
            "Could not convert account1 address string {} into account address.",
            account1_address_string
        ));
    let account1_balance = test_runner.get_component_balance(account1_address, XRD);
    println!(
        "Account1 balance for account {}: {:?}",
        account1_address_string, account1_balance
    );
    let account2_address_string = account_addresses[1].clone().0;
    let account2_address =
        ComponentAddress::try_from_hex(&account2_address_string).expect(&format!(
            "Could not convert account2 address string {} into account address.",
            account2_address_string
        ));
    let account2_balance = test_runner.get_component_balance(account2_address, XRD);
    println!(
        "Account2 balance for account {}: {:?}",
        account2_address_string, account2_balance
    );
}

#[test]
pub fn change_dapp_def_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (_component_address, dapp_def_address) =
        setup_component(&main_account, XRD, dextr_admin_token, &mut test_runner);

    // change metadata with authorisation - should succeed
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .set_metadata(dapp_def_address, "name", "DeXter Claim Component 2")
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();

    // change metadata without authorisation = should fail
    let tx_manifest = ManifestBuilder::new()
        .set_metadata(dapp_def_address, "name", "DeXter Claim Component 2")
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_failure();
}

#[test]
pub fn change_admin_role_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let _second_account = test_runner.new_allocated_account();
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(2), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address) =
        setup_component(&main_account, XRD, dextr_admin_token, &mut test_runner);

    // change admin role without authorisation - should fail
    let tx_manifest = ManifestBuilder::new()
        .set_main_role(
            component_address.clone(),
            "admin",
            rule!(require_amount(2, dextr_admin_token.clone())),
        )
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_failure();

    // change admin role with authorisation - should succeed
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .set_main_role(
            component_address.clone(),
            "admin",
            rule!(require_amount(2, dextr_admin_token.clone())),
        )
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();

    // try to run admins-only protected method - should fail
    let (test_str, _test_accounts) = build_accounts_test_str(&mut test_runner);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2, XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(
                    String::from(test_str.trim()),
                    vec!(lookup.bucket("xrd_bucket"))
                ),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_failure();

    // try to run admins-only protected method again with proper authorisation - should succeed
    let (test_str, _test_accounts) = build_accounts_test_str(&mut test_runner);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2, XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 2)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(
                    String::from(test_str.trim()),
                    vec!(lookup.bucket("xrd_bucket"))
                ),
            )
        })
        .drop_all_proofs()
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
}

fn build_accounts_test_str(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> (String, Vec<(String, Secp256k1PublicKey, Decimal)>) {
    println!("Starting to create test str...");
    let mut account_addresses: Vec<(String, Secp256k1PublicKey, Decimal)> = vec![];
    let xrd_string = XRD.to_hex();
    let (pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // test_runner.load_account_from_faucet(account1_address);
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey1, account1_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account1_address_string = String::from(account1_address.to_hex());
    account_addresses.push((account1_address_string.clone(), pubkey1, dec!("10357.79")));
    let (pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // test_runner.load_account_from_faucet(account2_address);
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account2_address_string = String::from(account2_address.to_hex());
    account_addresses.push((account2_address_string.clone(), pubkey2, dec!("10801.67")));
    let rewards_string = format!(
        r##"
    {{
        'reward_names': [
            [1, 'Liquidity Rewards'],
            [2, 'Trading Rewards']
        ],
        'tokens': [
            [1, '{xrd_string}']
        ],
        'accounts': [
            [
                '{account1_address_string}', [
                    [1, [[1, '123.34']]],
                    [2, [[1, '234.45']]]
                ]
            ],
            [
                '{account2_address_string}', [
                    [1, [[1, '345.67']]],
                    [2, [[1, '456']]]
                ]
            ]
        ],
        'orders': []
    }}
    "##
    );
    let trimmed_rewards_string = rewards_string
        .replace("\n", "")
        .replace("\r", "")
        .replace(" ", "")
        .clone();
    // println!(
    //     "Output string trimmed: {:?}",
    //     trimmed_rewards_string.clone()
    // );
    (trimmed_rewards_string, account_addresses)
}

fn build_remove_accounts_test_str(account_addresses: Vec<String>) -> String {
    println!("Starting to create remove accounts test str...");
    let xrd_string = XRD.to_hex();
    let account1_address = account_addresses[0].clone();
    let account2_address = account_addresses[1].clone();
    let rewards_string = format!(
        r##"
    {{
        'reward_names': [
            [1, 'Liquidity Rewards'],
            [2, 'Trading Rewards']
        ],
        'tokens': [
            [1, '{xrd_string}']
        ],
        'accounts': [
            [
                '{account1_address}', [
                    [1, [[1, '123.34']]],
                    [2, [[1, '134.45']]]
                ]
            ],
            [
                '{account2_address}', [
                    [1, [[1, '145.67']]],
                    [2, [[1, '456']]]
                ]
            ]
        ],
        'orders': []
    }}
    "##
    );
    let trimmed_rewards_string = rewards_string
        .replace("\n", "")
        .replace("\r", "")
        .replace(" ", "")
        .clone();
    // println!(
    //     "Output string trimmed: {:?}",
    //     trimmed_rewards_string.clone()
    // );
    trimmed_rewards_string
}

fn build_orders_test_str(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> String {
    println!("Starting to create test str...");
    let mut account_addresses: Vec<(String, Secp256k1PublicKey)> = vec![];
    let (pubkey1, _, account1_address) = test_runner.new_allocated_account();
    test_runner.load_account_from_faucet(account1_address);
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey1, account1_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account1_address_string = String::from(account1_address.to_hex());
    account_addresses.push((account1_address_string.clone(), pubkey1));
    let (pubkey2, _, account2_address) = test_runner.new_allocated_account();
    test_runner.load_account_from_faucet(account2_address);
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account2_address_string = String::from(account2_address.to_hex());
    account_addresses.push((account2_address_string.clone(), pubkey2));
    let rewards_string = format!(
        r##"
    {{
        'orders': [
            {{
                'pair_receipt_address': 'pair1_address',
                'pair_rewards': [
                    [1, '123.45'],
                    [2, '234.56']
                ]
            }},
            {{
                'pair_receipt_address': 'pair2_address',
                'pair_rewards': [
                    [1, '345.67'],
                    [2, '456.78']
                ]
            }}
        ]
    }}
    "##
    );
    let trimmed_rewards_string = rewards_string
        .replace("\n", "")
        .replace("\r", "")
        .replace(" ", "")
        .clone();
    // println!(
    //     "Output string trimmed: {:?}",
    //     trimmed_rewards_string.clone()
    // );
    trimmed_rewards_string
}

fn setup_component(
    main_account: &(Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress),
    dextr_token: ResourceAddress,
    dextr_admin_token: ResourceAddress,
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> (ComponentAddress, ComponentAddress) {
    let package_address = test_runner.compile_and_publish(this_package!());

    let tx_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "DexterClaimComponent",
            "new",
            manifest_args!(dextr_token, dextr_admin_token),
        )
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let result = receipt.expect_commit_success();

    // println!(
    //     "New component addresses: {:?}",
    //     result.new_component_addresses()
    // );
    let claim_component_address = result.new_component_addresses()[0];
    let dapp_def_address = result.new_component_addresses()[1];
    // println!("Claim component address: {:?}", claim_component_address);
    (claim_component_address, dapp_def_address)
}
