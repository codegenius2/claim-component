use json::JsonValue;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone, Debug)]
pub struct AdminData {
    pub name: String,
}
#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonRewardsData {
    pub reward_names: Vec<JsonRewardNames>,
    pub tokens: Vec<JsonRewardTokens>,
    pub accounts: Vec<JsonAccountRewards>,
    pub orders: Vec<JsonPairOrderRewards>,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonRewardNames {
    pub name_id: u64,
    pub name: String,
}
#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonRewardTokens {
    pub token_id: u64,
    pub token_address: String,
}
#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonAccountRewards {
    pub account_address: String,
    pub account_rewards: Vec<JsonNameRewards>,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonNameRewards {
    pub name_id: u64,
    pub name_rewards: Vec<JsonNameTokenRewards>,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonNameTokenRewards {
    pub token_id: u64,
    pub token_reward: Decimal,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonPairOrderRewards {
    pub pair_receipt_address: String,
    pub pair_rewards: Vec<(u64, Decimal)>,
}

#[blueprint]
mod dexter_claim_component {
    enable_method_auth! {
        roles {
            super_admin => updatable_by: [OWNER];
            admin => updatable_by: [OWNER];
        },
        methods {
            add_rewards => restrict_to: [admin];
            claim_rewards => PUBLIC;
        }
    }
    struct DexterClaimComponent {
        dextr_token_address: String,
        admin_token_address: ResourceAddress,
        claim_accounts: KeyValueStore<String, HashMap<String, HashMap<String, Decimal>>>,
        claim_orders: KeyValueStore<String, Decimal>, // KVS<Order receipt resource address +"#"+ Order recipt local id, Reward Amount>
        claim_vaults: KeyValueStore<String, Vault>,
        env: String,
    }

    impl DexterClaimComponent {
        pub fn new(
            dextr_token_address: ResourceAddress,
            admin_token_address: ResourceAddress,
        ) -> Global<DexterClaimComponent> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<DexterClaimComponent>::blueprint_id());
            // let require_component_rule = rule!(require(global_caller(component_address)));
            // set up a dapp definition account for the pair
            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None);
            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());
            // metadata and owner for the dapp definition are added later in the function after the entities are created.

            let new_component = Self {
                dextr_token_address: DexterClaimComponent::create_resource_address_string(
                    &dextr_token_address,
                    "local",
                ),
                admin_token_address,
                claim_accounts: KeyValueStore::new(),
                claim_orders: KeyValueStore::new(),
                claim_vaults: KeyValueStore::new(),
                env: String::from("local"),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(admin_token_address.clone()))))
            .with_address(address_reservation)
            .roles(roles!(
                super_admin => rule!(require(admin_token_address.clone()));
                admin => rule!(require(admin_token_address.clone()));
            ))
            .metadata(metadata! {
              init {
                "name" => String::from("DeXter Claim Component"), updatable;
                "description" => String::from("DeXter Liquidity and Trading Rewards Claim Component."), updatable;
                "tags" => vec!["DeXter"], updatable;
                "dapp_definitions" => vec![dapp_def_address.clone()], updatable;
              }
            })
            .globalize();

            // set dapp definition metadata and owner
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account.set_metadata("name", format!("DeXter Claim Component"));
            dapp_def_account.set_metadata(
                "description",
                format!("A component to facilitate the distribution of rewards to the DeXter community."),
            );
            dapp_def_account.set_metadata(
                "icon_url",
                Url::of("https://dexteronradix.com/logo_icon.svg"),
            );
            dapp_def_account.set_metadata(
                "claimed_entities",
                vec![GlobalAddress::from(component_address.clone())],
            );
            dapp_def_account.set_owner_role(rule!(require(admin_token_address.clone())));

            new_component
        }

        pub fn add_rewards(
            &mut self,
            rewards_data_string: String,
            rewards_buckets: Vec<Bucket>,
        ) -> Vec<Bucket> {
            // comment below out for production
            for reward_bucket in rewards_buckets.iter() {
                let _reward_bucket_address_string =
                    DexterClaimComponent::create_resource_address_string(
                        &reward_bucket.resource_address(),
                        &self.env,
                    );
                info!(
                    "Reward bucket for resource {}: Amount: {}",
                    _reward_bucket_address_string,
                    reward_bucket.amount()
                );
            }
            // comment above out for production

            let extracted_data = self.parse_rewards_data(rewards_data_string);
            info!("rewards data: {:?}", &extracted_data);
            let token_rewards = self.load_rewards_data(&extracted_data, true);
            let mut return_buckets: Vec<Bucket> = vec![];
            for mut token_bucket in rewards_buckets {
                // let token_bucket_address = token_bucket.resource_address();
                let token_bucket_address_string =
                    DexterClaimComponent::create_resource_address_string(
                        &token_bucket.resource_address(),
                        &self.env,
                    );
                if let Some(token_reward) = token_rewards.get(&token_bucket_address_string) {
                    if self
                        .claim_vaults
                        .get(&token_bucket_address_string)
                        .is_some()
                    {
                        let mut claim_vault = self
                            .claim_vaults
                            .get_mut(&token_bucket_address_string)
                            .unwrap();
                        claim_vault.put(token_bucket.take(token_reward.clone()));
                    } else {
                        let new_vault = Vault::with_bucket(token_bucket.take(token_reward.clone()));
                        self.claim_vaults
                            .insert(token_bucket_address_string, new_vault);
                    }
                }
                return_buckets.push(token_bucket);
            }

            // comment below out for production
            for return_bucket in return_buckets.iter() {
                let _return_bucket_address_string =
                    DexterClaimComponent::create_resource_address_string(
                        &return_bucket.resource_address(),
                        &self.env,
                    );
                info!(
                    "Return bucket for resource {}: Amount: {}",
                    _return_bucket_address_string,
                    return_bucket.amount()
                );
            }
            // comment above out for production

            return_buckets
        }

        pub fn claim_rewards(
            &mut self,
            accounts: Vec<ComponentAddress>,
            orders_proofs: Vec<NonFungibleProof>,
        ) -> Bucket {
            info!("Starting to claim rewards!");
            let mut accounts_to_remove: Vec<String> = vec![];
            for account in accounts {
                let mut account_returned_buckets: Vec<Bucket> = vec![];
                let account_address_string =
                    DexterClaimComponent::create_component_address_string(&account, &self.env);
                let mut global_account: Global<Account> = account.into();
                if let Some(account_data) = self.claim_accounts.get(&account_address_string) {
                    info!(
                        "Claiming rewards for account: {} rewards: {:?}",
                        account_address_string.clone(),
                        &account_data.to_owned()
                    );
                    for (_name_string, name_rewards) in account_data.iter() {
                        info!("Claiming rewards for name {:?}", _name_string);
                        for (token_address_string, token_reward_amount) in name_rewards.iter() {
                            let token_address =
                                DexterClaimComponent::create_resource_address_from_string(
                                    token_address_string.clone(),
                                    &self.env,
                                );
                            if account_returned_buckets
                                .iter()
                                .find(|bucket| bucket.resource_address() == token_address)
                                .is_none()
                            {
                                account_returned_buckets.push(Bucket::new(token_address));
                            }
                            let token_bucket = account_returned_buckets.iter_mut().find(|bucket| {bucket.resource_address() == token_address}).expect(&format!("Could not find account return bucket for token with resource address: {:?}", token_address));
                            // let token_bucket = returned_buckets.entry(token_address).or_insert(Bucket::new(token_address));
                            info!(
                                "Token bucket amount before taking out reward: {:?}",
                                token_bucket.amount()
                            );
                            let mut token_vault = self
                                .claim_vaults
                                .get_mut(&token_address_string)
                                .expect(&format!(
                                    "Could not find vault for reward token {:?}",
                                    token_address_string
                                ));
                            info!(
                                "Token vault amount before taking out reward: {:?}",
                                token_vault.amount()
                            );
                            token_bucket.put(token_vault.take(token_reward_amount.clone()));
                            info!(
                                "Token vault amount after taking out reward: {:?}",
                                token_vault.amount()
                            );
                            info!(
                                "Token bucket amount after taking out reward: {:?}",
                                token_bucket.amount()
                            );
                        }
                    }
                    global_account.try_deposit_batch_or_abort(account_returned_buckets, None);
                    accounts_to_remove.push(account_address_string);
                }
            }
            info!("Before removing accounts");
            for account_to_remove in accounts_to_remove {
                // as a temporary measure, instead of removing account just give it an empty entry. The gateway does not handle removed keys in a kVS well at the moment.
                self.claim_accounts
                    .insert(account_to_remove, HashMap::new());
                // self.claim_accounts.remove(&account_to_remove);
            }
            info!("Handled accounts claims");

            info!("Starting to handle order claims");
            let mut dextr_return_bucket =
                Bucket::new(DexterClaimComponent::create_resource_address_from_string(
                    self.dextr_token_address.clone(),
                    &self.env,
                ));
            let mut orders_to_remove: Vec<String> = vec![];
            if let Some(mut dextr_vault) = self.claim_vaults.get_mut(&self.dextr_token_address) {
                for orders_proof in orders_proofs {
                    let proof_resource_address = orders_proof.resource_address();
                    let order_ids = orders_proof.skip_checking().non_fungible_local_ids();
                    for order_id in order_ids {
                        let mut order_index_string =
                            DexterClaimComponent::create_resource_address_string(
                                &proof_resource_address,
                                &self.env,
                            );
                        order_index_string.push_str("#");
                        let order_id_string = order_id.to_string();
                        info!("order_id string: {:?}", order_id_string);
                        order_index_string.push_str(&order_id_string);
                        info!("Order_index_string {:?}", order_index_string);
                        if let Some(order_claim_amount) = self.claim_orders.get(&order_index_string)
                        {
                            dextr_return_bucket.put(dextr_vault.take(order_claim_amount.clone()));
                            orders_to_remove.push(order_index_string.clone());
                        }
                    }
                }
            }
            for order in orders_to_remove {
                self.claim_orders.remove(&order);
            }
            dextr_return_bucket
        }

        fn parse_rewards_data(&self, rewards_data_str: String) -> JsonRewardsData {
            let mut result = JsonRewardsData {
                reward_names: vec![],
                tokens: vec![],
                accounts: vec![],
                orders: vec![],
            };
            let changed_rewards_data_str = rewards_data_str.replace("'", "\"");
            let extracted_data =
                json::parse(&changed_rewards_data_str).expect("Invalid JSON specified!");
            info!("Extracted JSON data: {:?}", extracted_data);
            if let JsonValue::Object(rewards_data_obj) = extracted_data {
                for (field_key, field_value) in rewards_data_obj.iter() {
                    match field_key {
                        "reward_names" => {
                            if let JsonValue::Array(names_data) = field_value {
                                for name_data in names_data {
                                    let name_id = self
                                        .get_number_value(&name_data[0].clone(), "Reward Name Id");
                                    let name =
                                        self.get_string_value(&name_data[1].clone(), "Reward Name");
                                    result.reward_names.push(JsonRewardNames { name_id, name });
                                }
                            } else {
                                panic!("\"tokens\" field must be an array");
                            }
                        }
                        "tokens" => {
                            // info!("Parsing tokens data: {:?}", field_value);
                            if let JsonValue::Array(tokens_data) = field_value {
                                for token_data in tokens_data {
                                    // info!("Parsing token data: {:?}", token_data);
                                    let token_id =
                                        self.get_number_value(&token_data[0].clone(), "Token Id");
                                    let token_address = self
                                        .get_string_value(&token_data[1].clone(), "Token Address");
                                    result.tokens.push(JsonRewardTokens {
                                        token_id,
                                        token_address,
                                    });
                                }
                            } else {
                                panic!("\"tokens\" field must be an array");
                            }
                        }
                        "accounts" => {
                            info!("Parsing accounts data: {:?}", field_value);
                            if let JsonValue::Array(accounts_data) = field_value {
                                for account_data in accounts_data {
                                    info!("Parsing account data: {:?}", account_data);
                                    if let JsonValue::Array(account_base_data) = account_data {
                                        let account_address = self.get_string_value(
                                            &account_base_data[0].clone(),
                                            "Account Address",
                                        );
                                        let mut account_name_rewards: Vec<JsonNameRewards> = vec![];
                                        if let JsonValue::Array(account_name_rewards_data) =
                                            account_base_data[1].clone()
                                        {
                                            for name_reward_data in account_name_rewards_data {
                                                info!(
                                                    "Parsing name rewards data: {:?}",
                                                    name_reward_data
                                                );
                                                let name_id = self.get_number_value(
                                                    &name_reward_data[0].clone(),
                                                    "Name Id",
                                                );
                                                let mut name_token_rewards: Vec<
                                                    JsonNameTokenRewards,
                                                > = vec![];
                                                if let JsonValue::Array(name_token_rewards_data) =
                                                    name_reward_data[1].clone()
                                                {
                                                    for name_token_reward in name_token_rewards_data
                                                    {
                                                        let reward_token = self.get_number_value(
                                                            &name_token_reward[0],
                                                            "Token Id",
                                                        );
                                                        let reward_amount_str = self
                                                            .get_string_value(
                                                                &name_token_reward[1],
                                                                "Reward Amount",
                                                            );
                                                        let reward_amount = Decimal::try_from(reward_amount_str.to_owned()).expect(&format!("Could not convert reward amount {:?} to Decimal. ", reward_amount_str));
                                                        name_token_rewards.push(
                                                            JsonNameTokenRewards {
                                                                token_id: reward_token,
                                                                token_reward: reward_amount,
                                                            },
                                                        );
                                                    }
                                                } else {
                                                    panic!("Rewards data for account {} and reward name id {} is not specified as an array.", account_address, name_id);
                                                }
                                                account_name_rewards.push(JsonNameRewards {
                                                    name_id,
                                                    name_rewards: name_token_rewards,
                                                });
                                            }
                                        } else {
                                            panic!("Account rewards per name not specified as an array for account {}.", account_address);
                                        }
                                        info!(
                                            "Account extracted data: ({:?}, {:?})",
                                            account_address, account_name_rewards
                                        );
                                        result.accounts.push(JsonAccountRewards {
                                            account_address,
                                            account_rewards: account_name_rewards,
                                        });
                                    } else {
                                        panic!(
                                            "Account data not specified as an array. : {:?}",
                                            account_data
                                        );
                                    }
                                }
                            } else {
                                panic!("\"accounts\" field must be an array");
                            }
                        }
                        "orders" => {
                            info!("Parsing orders data: {:?}", field_value);
                            if let JsonValue::Array(orders_data) = field_value {
                                for pair_data_obj in orders_data {
                                    if let JsonValue::Object(pair_data) = pair_data_obj {
                                        info!("Pair data: {:?}", pair_data);
                                        let mut pair_receipt_address = String::from("");
                                        let mut pair_rewards: Vec<(u64, Decimal)> = vec![];
                                        for (field_key, field_value) in pair_data.iter() {
                                            info!(
                                                "Pair data field key: {:?} => value {:?}",
                                                field_key, field_value
                                            );
                                            match field_key {
                                                "pair_receipt_address" => {
                                                    pair_receipt_address = self
                                                        .get_string_value(field_value, field_key);
                                                }
                                                "pair_rewards" => {
                                                    if let JsonValue::Array(pair_order_rewards) =
                                                        field_value
                                                    {
                                                        info!(
                                                            "Pair rewards: {:?}",
                                                            pair_order_rewards
                                                        );
                                                        for order_reward_data in pair_order_rewards
                                                        {
                                                            if let JsonValue::Array(
                                                                temp_order_reward_data,
                                                            ) = order_reward_data
                                                            {
                                                                let order_id = self
                                                                    .get_number_value(
                                                                        &temp_order_reward_data[0]
                                                                            .clone(),
                                                                        "Order Id",
                                                                    );
                                                                // let order_id = order_id_str.parse::<u64>().expect(&format!("Could not convert order id to u64: {}", order_id_str));
                                                                let order_reward_str = self
                                                                    .get_string_value(
                                                                        &temp_order_reward_data[1]
                                                                            .clone(),
                                                                        "Order Reward",
                                                                    );
                                                                let order_reward = Decimal::try_from(order_reward_str.to_owned()).expect(&format!("Could not convert reward amount {:?} to Decimal. ", order_reward_str));
                                                                pair_rewards
                                                                    .push((order_id, order_reward));
                                                            } else {
                                                                panic!("Order reward data must be an array");
                                                            }
                                                        }
                                                    } else {
                                                        panic!("pair rewards must be an array");
                                                    }
                                                }
                                                _ => {
                                                    panic!(
                                                        "Unknown field \"{}\" in pair rewards data",
                                                        field_key
                                                    )
                                                }
                                            }
                                        }
                                        info!("Loaded pair rewards: {:?}", pair_rewards);
                                        if pair_receipt_address == "" {
                                            panic!("Found pair rewards without preceding pair address.");
                                        } else {
                                            result.orders.push(JsonPairOrderRewards {
                                                pair_receipt_address,
                                                pair_rewards,
                                            })
                                        }
                                    } else {
                                        panic!("Pair Orders data must be an object.");
                                    }
                                }
                            }
                        }
                        _ => {
                            panic!("Unknown field \"{}\" in rewards data", field_key);
                        }
                    }
                }
            } else {
                panic!("Rewards data is not an object");
            }
            result
        }

        fn load_rewards_data(
            &mut self,
            rewards_data: &JsonRewardsData,
            add: bool,
        ) -> HashMap<String, Decimal> {
            let mut names_map: HashMap<u64, String> = HashMap::new();
            for name_data in rewards_data.reward_names.clone() {
                names_map.insert(name_data.name_id, name_data.name);
            }
            let mut tokens_map: HashMap<u64, (String, Decimal)> = HashMap::new();
            for token_data in rewards_data.tokens.clone() {
                tokens_map.insert(
                    token_data.token_id,
                    (token_data.token_address, Decimal::ZERO),
                );
            }
            // process accounts rewards
            for account_data in rewards_data.accounts.clone() {
                let mut skip_rest = false;
                let account_address_str = account_data.account_address.clone();
                info!("Account address string: {:?}", account_address_str);
                let _account_address = DexterClaimComponent::create_component_address_from_string(
                    account_address_str.clone(),
                    &self.env,
                );
                info!("Account address: {:?}", _account_address);
                let mut existing_account_data: HashMap<String, HashMap<String, Decimal>>;
                if self.claim_accounts.get(&account_address_str).is_some() {
                    existing_account_data = self
                        .claim_accounts
                        .get(&account_address_str)
                        .unwrap()
                        .to_owned();
                } else if !add {
                    skip_rest = true;
                    existing_account_data = HashMap::new();
                } else {
                    info!("Inserting new account...");
                    existing_account_data = HashMap::new();
                }
                if !skip_rest {
                    for name_rewards_data in account_data.account_rewards {
                        info!("Name Rewards Data: {:?}", name_rewards_data);
                        let reward_name = names_map
                            .get(&name_rewards_data.name_id)
                            .expect(&format!(
                                "Could not find reward name with id {} in reward names data.",
                                name_rewards_data.name_id
                            ))
                            .to_owned();
                        let mut existing_name_data = existing_account_data
                            .entry(reward_name.clone())
                            .or_insert(HashMap::new())
                            .clone();
                        info!("Existing name data: {:?}", existing_name_data);
                        for token_reward_data in name_rewards_data.name_rewards {
                            info!("Token Reward Data: {:?}", token_reward_data);
                            let tokens_map_data =
                                tokens_map.get(&token_reward_data.token_id).expect(&format!(
                                    "Could not find token with id {} in tokens data.",
                                    token_reward_data.token_id
                                ));
                            info!("Tokens Map data for token: {:?}", tokens_map_data);

                            let token_address = tokens_map_data.0.clone();

                            let mut existing_token_total = existing_name_data
                                .entry(token_address.clone())
                                .or_insert(Decimal::ZERO)
                                .clone();
                            let mut token_change = token_reward_data.token_reward.clone();
                            if add {
                                existing_token_total =
                                    existing_token_total.checked_add(token_change).expect(
                                        "Could not add new token reward to existing token total",
                                    );
                            } else {
                                token_change =
                                    existing_token_total.min(token_reward_data.token_reward);
                                existing_token_total =
                                    existing_token_total.checked_sub(token_change).expect(
                                        "Could not remove token reward from existing token total",
                                    );
                            }
                            if existing_token_total > Decimal::ZERO {
                                existing_name_data
                                    .insert(token_address.clone(), existing_token_total.to_owned());
                            } else {
                                existing_name_data.remove(&token_address.clone());
                            }
                            info!("Existing name data (after update) {:?}", existing_name_data);
                            let total_token_change = tokens_map_data.1 + token_change;
                            tokens_map.insert(
                                token_reward_data.token_id,
                                (token_address.clone(), total_token_change),
                            );
                            info!("Total token reward: {:?}", total_token_change);
                        }
                        if existing_name_data.len() > 0 {
                            existing_account_data
                                .insert(reward_name, existing_name_data.to_owned());
                        } else {
                            existing_account_data.remove(&reward_name);
                        }
                        info!("Existing account data: {:?}", existing_account_data);
                    }
                    // At the moment the gateway does not show an account that has been removed and then later added back again.
                    // So the component will keep an entry for an account, even if it is empty to make sure it can be picked up by the gateway.
                    self.claim_accounts
                        .insert(account_address_str, existing_account_data);
                }
            }
            let mut token_totals_map: HashMap<String, Decimal> = HashMap::new();
            for value in tokens_map.values() {
                token_totals_map.insert(value.0.clone(), value.1.clone());
            }

            // process orders rewards
            let mut total_orders_reward_amount = Decimal::ZERO;
            for order_pair_data in &rewards_data.orders {
                let pair_address_string = order_pair_data.pair_receipt_address.clone();
                for (order_id, order_reward_amount) in &order_pair_data.pair_rewards {
                    let mut order_id_string = pair_address_string.clone();
                    order_id_string.push_str("#");
                    order_id_string.push_str(&order_id.to_string());
                    info!("Order id string: {:?}", order_id_string);
                    total_orders_reward_amount = total_orders_reward_amount
                        .checked_add(order_reward_amount.clone())
                        .expect("Could not add to total_orders_reward_amount.");
                    self.claim_orders
                        .insert(order_id_string, order_reward_amount.clone());
                }
            }
            let mut dextr_token_total = token_totals_map
                .entry(self.dextr_token_address.clone())
                .or_insert(Decimal::ZERO)
                .clone();
            dextr_token_total = dextr_token_total
                .checked_add(total_orders_reward_amount)
                .unwrap();
            token_totals_map.insert(self.dextr_token_address.clone(), dextr_token_total.clone());
            info!(
                "Token Totals Map after rewards load: {:?}",
                token_totals_map
            );
            token_totals_map
        }

        fn get_string_value(&self, json_string_value: &JsonValue, field_name: &str) -> String {
            match json_string_value {
                JsonValue::Short(field_value) => {
                    // info!("{}: {:?}", field_name, field_value);
                    return field_value.to_string();
                }
                JsonValue::String(field_value) => {
                    // info!("{}: {:?}", field_name, field_value);
                    return field_value.to_string();
                }
                _ => {
                    panic!("{} must be a string", field_name);
                }
            }
        }

        fn get_number_value(&self, json_value: &JsonValue, field_name: &str) -> u64 {
            info!("Get number value for {:?}", json_value);
            match json_value {
                JsonValue::Number(field_value) => {
                    info!("{}: {:?}", field_name, field_value);
                    let field_number = field_value
                        .as_fixed_point_u64(0)
                        .expect(&format!("Could not convert {} to a u64", field_value));
                    info!("Field number: {}", field_number);
                    return field_number;
                }
                JsonValue::Short(field_value) => {
                    // info!("{}: {:?}", field_name, field_value);
                    return field_value
                        .parse::<u64>()
                        .expect(&format!("Could not convert {} to a u64", field_value));
                }
                JsonValue::String(field_value) => {
                    // info!("{}: {:?}", field_name, field_value);
                    return field_value
                        .parse::<u64>()
                        .expect(&format!("Could not convert {} to a u64", field_value));
                }
                _ => {
                    panic!(
                        "{} must be a valid number or number string: {:?}",
                        field_name, json_value
                    );
                }
            }
        }

        fn create_resource_address_string(address: &ResourceAddress, env: &str) -> String {
            if env == "mainnet" || env == "stokenet" {
                Runtime::bech32_encode_address(address.clone())
            } else {
                address.to_hex()
            }
        }

        fn create_resource_address_from_string(string: String, env: &str) -> ResourceAddress {
            if env == "mainnet" {
                ResourceAddress::try_from_bech32(
                    &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
                    &string,
                )
                .expect(&format!(
                    "Could not convert string {:?} into ResourceAddress.",
                    string
                ))
            } else if env == "stokenet" {
                let stokenet_def = NetworkDefinition {
                    id: 2,
                    logical_name: String::from("stokenet"),
                    hrp_suffix: String::from("tdx_2_"),
                };
                ResourceAddress::try_from_bech32(&AddressBech32Decoder::new(&stokenet_def), &string)
                    .expect(&format!(
                        "Could not convert string {:?} into ResourceAddress.",
                        string
                    ))
            } else {
                ResourceAddress::try_from_hex(&string).expect(&format!(
                    "Could not convert string {:?} into ResourceAddress.",
                    string
                ))
            }
        }

        fn create_component_address_string(address: &ComponentAddress, env: &str) -> String {
            if env == "mainnet" || env == "stokenet" {
                Runtime::bech32_encode_address(address.clone())
            } else {
                address.to_hex()
            }
        }

        fn create_component_address_from_string(string: String, env: &str) -> ComponentAddress {
            if env == "mainnet" {
                ComponentAddress::try_from_bech32(
                    &AddressBech32Decoder::new(&NetworkDefinition::mainnet()),
                    &string,
                )
                .expect(&format!(
                    "Could not convert string {:?} into ComponentAddress.",
                    string
                ))
            } else if env == "stokenet" {
                let stokenet_def = NetworkDefinition {
                    id: 2,
                    logical_name: String::from("stokenet"),
                    hrp_suffix: String::from("tdx_2_"),
                };
                ComponentAddress::try_from_bech32(
                    &AddressBech32Decoder::new(&stokenet_def),
                    &string,
                )
                .expect(&format!(
                    "Could not convert string {:?} into ComponentAddress.",
                    string
                ))
            } else {
                ComponentAddress::try_from_hex(&string).expect(&format!(
                    "Could not convert string {:?} into ComponentAddress.",
                    string
                ))
            }
        }
    }
}
