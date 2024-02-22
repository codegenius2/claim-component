// use scrypto::prelude::DIVISIBILITY_MAXIMUM;
use scrypto::prelude::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;
use transaction::prelude::ManifestBuilder;

#[test]
fn setup_component_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    setup_component(&main_account, XRD, &mut test_runner);
}

#[test]
fn add_accounts_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let dextr_token = XRD;
    // let dextr_token =
    //     test_runner.create_fungible_resource(dec!("10000"), DIVISIBILITY_MAXIMUM, main_account.2);
    let component_address = setup_component(&main_account, dextr_token, &mut test_runner);
    // let test_str2 = r##"{"accounts":[["account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma","756.94"]],"orders":[{ "pair_address": "DEXTR/XRD", "pair_rewards": [["1303","1153.12"],["1306","14089.93"]]}]}"##;
    let (test_str, account_addresses) = build_accounts_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), dextr_token, dec!("2000"))
        .take_all_from_worktop(dextr_token, "dextr_bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(String::from(test_str), vec!(lookup.bucket("dextr_bucket"))),
            )
        })
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

    let test_account_address = account_addresses[0].clone();
    println!("Test account address: {:?}", test_account_address);
}

#[test]
fn add_orders_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    let component_address = setup_component(&main_account, XRD, &mut test_runner);
    // let test_str2 = r##"{"accounts":[["account_sim1c956qr3kxlgypxwst89j9yf24tjc7zxd4up38x37zr6q4jxdx9rhma","756.94"]],"orders":[{ "pair_address": "DEXTR/XRD", "pair_rewards": [["1303","1153.12"],["1306","14089.93"]]}]}"##;
    let test_str = build_orders_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2.clone(), XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .with_name_lookup(|builder, lookup| {
            builder.call_method(
                component_address,
                "add_rewards",
                manifest_args!(String::from(test_str), vec!(lookup.bucket("xrd_bucket"))),
            )
        })
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
}

#[test]
pub fn claim_accounts_rewards_test() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let main_account = test_runner.new_allocated_account();
    // test_runner.load_account_from_faucet(account1);
    let component_address = setup_component(&main_account, XRD, &mut test_runner);
    let (test_str, test_accounts) = build_accounts_test_str(&mut test_runner);
    println!("Test string: {:?}", test_str);
    let tx_manifest = ManifestBuilder::new()
        .withdraw_from_account(main_account.2, XRD, dec!("2000"))
        .take_all_from_worktop(XRD, "xrd_bucket")
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
    for (account_address_string, _) in test_accounts.clone() {
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

    let (test_account_string, test_account_pubkey) = test_accounts[1].clone();
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
    for (account_address_string, _) in test_accounts.clone() {
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

fn build_accounts_test_str(
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> (String, Vec<(String, Secp256k1PublicKey)>) {
    println!("Starting to create test str...");
    let mut account_addresses: Vec<(String, Secp256k1PublicKey)> = vec![];
    let xrd_string = XRD.to_hex();
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
    test_runner: &mut TestRunner<NoExtension, InMemorySubstateDatabase>,
) -> ComponentAddress {
    let package_address = test_runner.compile_and_publish(this_package!());

    let tx_manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "DexterClaimComponent",
            "new",
            manifest_args!(dextr_token),
        )
        .try_deposit_entire_worktop_or_abort(main_account.2, None)
        .build();

    let receipt = test_runner.execute_manifest_ignoring_fee(
        tx_manifest,
        vec![NonFungibleGlobalId::from_public_key(&main_account.0)],
    );
    // println!("Receipt: {:?}", receipt);
    let result = receipt.expect_commit_success();

    let claim_component_address = result.new_component_addresses()[0];
    // println!("Claim component address: {:?}", claim_component_address);
    claim_component_address
}
