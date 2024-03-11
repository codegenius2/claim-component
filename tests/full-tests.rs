// use claim_component::claim::dexter_claim_component::DexterClaimComponent;
use claim_component::claim::AccountRewardsData;
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
    let (component_address, _dapp_def_address, claim_token_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );

    let (_pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
    //     pubkey1, account1_address, account1_address_string
    // );
    let (_pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Liquidity Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("123.34")),
                        (account2_address, dec!("345.67"))
                    ),
                    lookup.bucket("dextr_bucket")
                ),
            )
        })
        .take_all_from_worktop(dextr_token, "dextr_bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("234.45")),
                        (account2_address, dec!("456"))
                    ),
                    lookup.bucket("dextr_bucket2")
                ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("234.45"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("456"),
        &claim_token_address,
        &mut test_runner,
    );
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
    let dextr_token = XRD;
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address, _claim_token_address) =
        setup_component(&main_account, XRD, dextr_admin_token, &mut test_runner);
    // let test_str2 = r##"{"accounts":[["account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma","756.94"]],"orders":[{ "pair_address": "DEXTR/XRD", "pair_rewards": [["1303","1153.12"],["1306","14089.93"]]}]}"##;
    let test_str = build_orders_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_orders_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    test_str,
                    lookup.bucket("dextr_bucket")
                ),
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
    // check_order_reward_amount(
    //     &component_address,
    //     "pair1_address",
    //     "1",
    //     dec!("123.45"),
    //     &mut test_runner,
    // );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), XRD);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8839.54"),
        "Expected Account Balance of 8839.54, but found {:?}",
        account_balance
    );
}

#[test]
pub fn add_liquidity_and_then_trading_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address, claim_token_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );

    let (_pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
    //     pubkey1, account1_address, account1_address_string
    // );
    let (_pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(
                    String::from("Liquidity Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("123.34")),
                        (account2_address, dec!("345.67"))
                    ),
                    String::from(""),
                    lookup.bucket("dextr_bucket")
                ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("9530.99"),
        "Expected Account Balance of 9530.99, but found {:?}",
        account_balance
    );
    let test_str = build_orders_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("234.45")),
                        (account2_address, dec!("456"))
                    ),
                    test_str,
                    lookup.bucket("dextr_bucket")
                ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("234.45"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("456"),
        &claim_token_address,
        &mut test_runner,
    );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), XRD);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("7680.08"),
        "Expected Account Balance of 7680.08, but found {:?}",
        account_balance
    );
}

#[test]
pub fn claim_accounts_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address, claim_token_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );

    let (_pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
    //     pubkey1, account1_address, account1_address_string
    // );
    let (_pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Liquidity Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("123.34")),
                        (account2_address, dec!("345.67"))
                    ),
                    lookup.bucket("dextr_bucket")
                ),
            )
        })
        .take_all_from_worktop(dextr_token, "dextr_bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("234.45")),
                        (account2_address, dec!("456"))
                    ),
                    lookup.bucket("dextr_bucket2")
                ),
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
    println!("After adding rewards...");
    let _result = receipt.expect_commit_success();
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    println!("After checking acount reward1");
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("234.45"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("456"),
        &claim_token_address,
        &mut test_runner,
    );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        account_balance
    );
    let test_account_address = account2_address.clone();
    let test_account_pubkey = _pubkey2.clone();
    let test_account_balance = dec!("10801.67");
    let order_proofs: Vec<ManifestProof> = vec![];
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            test_account_address.clone(),
            claim_token_address,
            vec![NonFungibleLocalId::string(test_account_address.to_hex()).unwrap()],
        )
        .pop_from_auth_zone("account_nft")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "claim_rewards",
                manifest_args!(vec!(lookup.proof("account_nft")), order_proofs),
            )
        })
        .try_deposit_entire_worktop_or_abort(test_account_address, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&test_account_pubkey)],
    );

    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(test_account_address, XRD);
    assert!(
        account_balance == test_account_balance,
        "Expected Account Balance of {:?}, but found {:?}",
        test_account_balance,
        account_balance
    );
    // for (account_address_string, _account_pubkey, _expected_account_balance) in
    //     test_accounts.clone()
    // {
    //     let account_address =
    //         ComponentAddress::try_from_hex(&account_address_string).expect(&format!(
    //             "Could not convert account address string {} into account address.",
    //             account_address_string
    //         ));
    //     let account_balance = test_runner.get_component_balance(account_address, XRD);
    //     println!(
    //         "Account balance for account {}: {:?}",
    //         account_address_string, account_balance
    //     );
    // }
}

#[test]
pub fn claim_accounts_with_two_nfts_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (component_address, _dapp_def_address, claim_token_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );

    let (_pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
    //     pubkey1, account1_address, account1_address_string
    // );
    let (_pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Liquidity Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("123.34")),
                        (account2_address, dec!("345.67"))
                    ),
                    lookup.bucket("dextr_bucket")
                ),
            )
        })
        .take_all_from_worktop(dextr_token, "dextr_bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("234.45")),
                        (account2_address, dec!("456"))
                    ),
                    lookup.bucket("dextr_bucket2")
                ),
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
    println!("After adding rewards...");
    let _result = receipt.expect_commit_success();
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    println!("After checking acount reward1");
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("234.45"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("456"),
        &claim_token_address,
        &mut test_runner,
    );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        account_balance
    );

    // ### Transfer claim token from Account 1 to Account 2
    let tx_manifest = ManifestBuilder::new()
        .call_method(
            account1_address.clone(),
            "withdraw_non_fungibles",
            manifest_args!(
                claim_token_address.clone(),
                vec![NonFungibleLocalId::string(account1_address.to_hex()).unwrap()],
            ),
        )
        .try_deposit_entire_worktop_or_abort(account2_address.clone(), None)
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&_pubkey1)],
    );
    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();

    // claim rewards for account2 - should include rewards for both accounts as claim nft for account1 is now also in account2
    let test_account_address = account2_address.clone();
    let test_account_pubkey = _pubkey2.clone();
    let test_account_balance = dec!("11159.46");
    let order_proofs: Vec<ManifestProof> = vec![];
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            test_account_address.clone(),
            claim_token_address,
            vec![
                NonFungibleLocalId::string(account1_address.to_hex()).unwrap(),
                NonFungibleLocalId::string(account2_address.to_hex()).unwrap(),
            ],
        )
        .pop_from_auth_zone("account_nft")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "claim_rewards",
                manifest_args!(vec!(lookup.proof("account_nft")), order_proofs),
            )
        })
        .try_deposit_entire_worktop_or_abort(test_account_address, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&test_account_pubkey)],
    );

    println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(test_account_address, XRD);
    assert!(
        account_balance == test_account_balance,
        "Expected Account Balance of {:?}, but found {:?}",
        test_account_balance,
        account_balance
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    println!("After checking acount reward1");
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
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
    let (component_address, _dapp_def_address, claim_token_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );

    let mut test_accounts: Vec<ComponentAddress> = vec![];
    let (_pubkey1, _, account1_address) = test_runner.new_allocated_account();
    test_accounts.push(account1_address.clone());
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
    //     pubkey1, account1_address, account1_address_string
    // );
    let (_pubkey2, _, account2_address) = test_runner.new_allocated_account();
    test_accounts.push(account2_address.clone());
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Liquidity Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("123.34")),
                        (account2_address, dec!("345.67"))
                    ),
                    lookup.bucket("dextr_bucket")
                ),
            )
        })
        .take_all_from_worktop(dextr_token, "dextr_bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("234.45")),
                        (account2_address, dec!("456"))
                    ),
                    lookup.bucket("dextr_bucket2")
                ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("234.45"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("456"),
        &claim_token_address,
        &mut test_runner,
    );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        account_balance
    );

    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .call_method(
            component_address,
            "remove_account_rewards",
            manifest_args!(
                String::from("Liquidity Rewards"),
                dextr_token.clone(),
                vec!(
                    (account1_address, dec!("123.34")),
                    (account2_address, dec!("145.67"))
                ),
            ),
        )
        .call_method(
            component_address,
            "remove_account_rewards",
            manifest_args!(
                String::from("Trading Rewards"),
                dextr_token.clone(),
                vec!(
                    (account1_address, dec!("134.45")),
                    (account2_address, dec!("456"))
                ),
            ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("100"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("200"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    let main_account_balance =
        test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Main Account balance: {:?}", main_account_balance);
    assert!(
        main_account_balance == dec!("9700"),
        "Expected Main Account Balance of 9700, but found {:?}",
        main_account_balance
    );
    let account1_balance = test_runner.get_component_balance(account1_address, XRD);
    println!(
        "Account1 balance for account {:?}: {:?}",
        account1_address, account1_balance
    );
    let account2_balance = test_runner.get_component_balance(account2_address, XRD);
    println!(
        "Account2 balance for account {:?}: {:?}",
        account2_address, account2_balance
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
    let (component_address, _dapp_def_address, claim_token_address) = setup_component(
        &main_account,
        dextr_token,
        dextr_admin_token,
        &mut test_runner,
    );

    let (_pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
    //     pubkey1, account1_address, account1_address_string
    // );
    let (_pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Liquidity Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("123.34")),
                        (account2_address, dec!("345.67"))
                    ),
                    lookup.bucket("dextr_bucket")
                ),
            )
        })
        .take_all_from_worktop(dextr_token, "dextr_bucket2")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_account_rewards",
                manifest_args!(
                    String::from("Trading Rewards"),
                    dextr_token.clone(),
                    vec!(
                        (account1_address, dec!("234.45")),
                        (account2_address, dec!("456"))
                    ),
                    lookup.bucket("dextr_bucket2")
                ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("123.34"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("234.45"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("345.67"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("456"),
        &claim_token_address,
        &mut test_runner,
    );
    let account_balance = test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Account balance: {:?}", account_balance);
    assert!(
        account_balance == dec!("8840.54"),
        "Expected Account Balance of 8840.54, but found {:?}",
        account_balance
    );

    let test_account_address = account2_address.clone();
    let test_account_pubkey = _pubkey2.clone();
    let test_account_balance = dec!("10801.67");
    let order_proofs: Vec<ManifestProof> = vec![];
    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_non_fungibles(
            test_account_address.clone(),
            claim_token_address,
            vec![NonFungibleLocalId::string(test_account_address.to_hex()).unwrap()],
        )
        .pop_from_auth_zone("account_nft")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "claim_rewards",
                manifest_args!(vec!(lookup.proof("account_nft")), order_proofs),
            )
        })
        .try_deposit_entire_worktop_or_abort(test_account_address, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&test_account_pubkey)],
    );
    // println!("Receipt: {:?}", receipt);
    let _result = receipt.expect_commit_success();
    let account_balance = test_runner.get_component_balance(test_account_address, XRD);

    println!("Claimed Account Balance: {:?}", account_balance);
    assert!(
        account_balance == test_account_balance,
        "Expected Account Balance of {:?}, but found {:?}",
        test_account_balance,
        account_balance
    );

    let tx_manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(main_account.2.clone(), dextr_admin_token, 1)
        .call_method(
            component_address,
            "remove_rewards",
            manifest_args!(
                String::from("Liquidity Rewards"),
                dextr_token.clone(),
                vec!(
                    (account1_address, dec!("123.34")),
                    (account2_address, dec!("145.67"))
                ),
                String::from("")
            ),
        )
        .call_method(
            component_address,
            "remove_rewards",
            manifest_args!(
                String::from("Trading Rewards"),
                dextr_token.clone(),
                vec!(
                    (account1_address, dec!("134.45")),
                    (account2_address, dec!("456"))
                ),
                String::from("")
            ),
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
    check_account_reward_amount(
        &account1_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account1_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("100"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Liquidity Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    check_account_reward_amount(
        &account2_address,
        String::from("Trading Rewards"),
        &dextr_token,
        dec!("0"),
        &claim_token_address,
        &mut test_runner,
    );
    let main_account_balance =
        test_runner.get_component_balance(main_account.2.clone(), dextr_token);
    println!("Main Account balance: {:?}", main_account_balance);
    assert!(
        main_account_balance == dec!("9098.33"),
        "Expected Main Account Balance of 9098.33, but found {:?}",
        main_account_balance
    );
    let account1_balance = test_runner.get_component_balance(account1_address, XRD);
    println!(
        "Account1 balance for account {:?}: {:?}",
        account1_address, account1_balance
    );
    let account2_balance = test_runner.get_component_balance(account2_address, XRD);
    println!(
        "Account2 balance for account {:?}: {:?}",
        account2_address, account2_balance
    );
}

#[test]
pub fn change_dapp_def_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_admin_token =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_NONE, main_account.2);
    let (_component_address, dapp_def_address, _claim_token_address) =
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
    let (component_address, _dapp_def_address, _claim_token_address) =
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

fn check_account_reward_amount(
    account_address: &ComponentAddress,
    reward_name: String,
    token_address: &ResourceAddress,
    expected_amount: Decimal,
    reward_nft_address: &ResourceAddress,
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) {
    let claim_token_data = test_runner.get_non_fungible_data::<AccountRewardsData>(
        reward_nft_address.clone(),
        NonFungibleLocalId::string(account_address.to_hex())
            .expect("Could not create NFT id from hex of account address"),
    );
    println!("Claim Token Data: {:?}", claim_token_data);
    let mut reward_amount = Decimal::ZERO;
    if let Some(account_name_rewards) = claim_token_data.rewards.get(&reward_name) {
        reward_amount = account_name_rewards
            .get(token_address)
            .expect("Could not find liquidity rewards for account 1 dextr token.")
            .to_owned();
    };
    assert!(
        reward_amount == expected_amount,
        "Reward amounts dont match. Expected {:?}, but found {:?}",
        expected_amount.clone(),
        reward_amount.clone()
    );
}

// fn check_order_reward_amount(
//     claim_component: &ComponentAddress,
//     pair_address_string: &str,
//     order_id: &str,
//     expected_amount: Decimal,
//     test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
// ) {
//     println!("Starting to check order reward...");
//     let component_state: DexterClaimComponent =
//         test_runner.component_state(claim_component.clone());
//     let claim_orders_kvs = component_state.claim_orders;
//     let full_order_id = format!("{}#{}#", pair_address_string, order_id);
//     println!("Full order id: {:?}", full_order_id);
//     // test_runner.get_kv_store_entry(claim_orders_kvs.)
//     let reward_amount = claim_orders_kvs
//         .get(&full_order_id.to_string())
//         .expect(&format!(
//             "Could not find order id {} in claim_orders kvs.",
//             full_order_id.clone()
//         ))
//         .clone();
//     println!("Order reward amount: {:?}", reward_amount.clone());
//     assert!(
//         reward_amount == expected_amount,
//         "Reward amounts dont match. Expected {:?}, but found {:?}",
//         expected_amount.clone(),
//         reward_amount.clone()
//     );
// }

fn build_accounts_test_str(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> (String, Vec<(String, Secp256k1PublicKey, Decimal)>) {
    println!("Starting to create test str...");
    let mut account_addresses: Vec<(String, Secp256k1PublicKey, Decimal)> = vec![];
    let xrd_string = XRD.to_hex();
    let (pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // test_runner.load_account_from_faucet(account1_address);
    let account1_address_string = account1_address.to_hex();
    println!(
        "New account created. Pub key: {:?}, Address: {:?}, Address hex: {:?}",
        pubkey1, account1_address, account1_address_string
    );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    account_addresses.push((account1_address_string.clone(), pubkey1, dec!("10357.79")));
    let (pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // test_runner.load_account_from_faucet(account2_address);
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account2_address_string = account2_address.to_hex();
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

// fn build_remove_accounts_test_str(account_addresses: Vec<String>) -> String {
//     println!("Starting to create remove accounts test str...");
//     let xrd_string = XRD.to_hex();
//     let account1_address = account_addresses[0].clone();
//     let account2_address = account_addresses[1].clone();
//     let rewards_string = format!(
//         r##"
//     {{
//         'reward_names': [
//             [1, 'Liquidity Rewards'],
//             [2, 'Trading Rewards']
//         ],
//         'tokens': [
//             [1, '{xrd_string}']
//         ],
//         'accounts': [
//             [
//                 '{account1_address}', [
//                     [1, [[1, '123.34']]],
//                     [2, [[1, '134.45']]]
//                 ]
//             ],
//             [
//                 '{account2_address}', [
//                     [1, [[1, '145.67']]],
//                     [2, [[1, '456']]]
//                 ]
//             ]
//         ],
//         'orders': []
//     }}
//     "##
//     );
//     let trimmed_rewards_string = rewards_string
//         .replace("\n", "")
//         .replace("\r", "")
//         .replace(" ", "")
//         .clone();
//     // println!(
//     //     "Output string trimmed: {:?}",
//     //     trimmed_rewards_string.clone()
//     // );
//     trimmed_rewards_string
// }

fn build_orders_test_str(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> String {
    println!("Starting to create test str...");
    let mut account_addresses: Vec<(String, Secp256k1PublicKey)> = vec![];
    let (pubkey1, _, account1_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey1, account1_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account1_address_string = String::from(account1_address.to_hex());
    account_addresses.push((account1_address_string.clone(), pubkey1));
    let (pubkey2, _, account2_address) = test_runner.new_allocated_account();
    // println!(
    //     "New account created. Pub key: {:?}, Address: {:?}",
    //     pubkey2, account2_address
    // );
    // let account_address_str = Runtime::bech32_encode_address(account_address);
    let account2_address_string = String::from(account2_address.to_hex());
    account_addresses.push((account2_address_string.clone(), pubkey2));
    let rewards_string = format!(
        r##"
    [
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
) -> (ComponentAddress, ComponentAddress, ResourceAddress) {
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
    let claim_token_address = result.new_resource_addresses()[0];
    (
        claim_component_address,
        dapp_def_address,
        claim_token_address,
    )
}
