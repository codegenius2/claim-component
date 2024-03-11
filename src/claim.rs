use json::JsonValue;
use scrypto::prelude::*;

#[derive(ScryptoSbor, NonFungibleData, Clone, Debug)]
pub struct AdminData {
    pub name: String,
}
#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonRewardsData {
    pub reward_name: String,
    pub token_address: String,
    pub accounts: Vec<JsonAccountRewards>,
    pub orders: Vec<JsonPairOrderRewards>,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonAccountRewards {
    pub account_address: String,
    pub account_reward: Decimal,
}

#[derive(ScryptoSbor, Clone, Debug)]
pub struct JsonPairOrderRewards {
    pub pair_receipt_address: String,
    pub pair_rewards: Vec<(u64, Decimal)>,
}

#[derive(ScryptoSbor, Clone, Debug, NonFungibleData)]
pub struct AccountRewardsData {
    pub account_address: String,
    #[mutable]
    pub rewards: HashMap<String, HashMap<ResourceAddress, Decimal>> // HashMap<Reward Name, HashMap<Token Address, Token Reward>>
}

#[blueprint]
#[types(AccountRewardsData, String, Decimal, Vault)]
mod dexter_claim_component {
    enable_method_auth! {
        roles {
            super_admin => updatable_by: [OWNER];
            admin => updatable_by: [OWNER];
        },
        methods {
            add_account_rewards => restrict_to: [admin];
            add_orders_rewards => restrict_to: [admin];
            add_rewards => restrict_to: [admin];
            remove_account_rewards => restrict_to: [super_admin];
            remove_orders_rewards => restrict_to: [super_admin];
            remove_rewards => restrict_to: [super_admin];
            claim_rewards => PUBLIC;
        }
    }
    struct DexterClaimComponent {
        pub dextr_token_address: ResourceAddress,
        pub admin_token_address: ResourceAddress,
        pub account_rewards_nft_manager: ResourceManager,
        // claim_accounts: KeyValueStore<String, HashMap<String, HashMap<String, Decimal>>>,
        pub claim_orders: KeyValueStore<String, Decimal>, // KVS<Order receipt resource address +"#"+ Order recipt local id, Reward Amount>
        pub claim_vaults: KeyValueStore<ResourceAddress, Vault>,
        pub env: String,
    }

    impl DexterClaimComponent {
        pub fn new(
            dextr_token_address: ResourceAddress,
            admin_token_address: ResourceAddress,
        ) -> Global<DexterClaimComponent> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<DexterClaimComponent>::blueprint_id());
            let require_component_rule = rule!(require(global_caller(component_address)));
            // set up a dapp definition account for the pair
            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None);
            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());
            // metadata and owner for the dapp definition are added later in the function after the entities are created.

            let account_rewards_nft_manager = 
                ResourceBuilder::new_string_non_fungible_with_registered_type::<AccountRewardsData>(OwnerRole::Updatable(rule!(require(admin_token_address.clone()))))
                .metadata(metadata! {
                    init{
                        "name" => "DeXter Rewards NFT", updatable;
                        "description" => "An NFT that keeps track of DeXter rewards related to an account. Although the NFT can be transferred between accounts, the rewards in this NFT will always relate to the specified account.", updatable;
                        "icon_url" => Url::of("https://dexteronradix.com/logo_icon.svg"), updatable;
                        "tags" => vec!["DeXter"], updatable;
                        "dapp_definitions" => vec![dapp_def_address.clone()], updatable;
                    }
                })
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater => require_component_rule.clone();
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .mint_roles(mint_roles! {
                    minter => require_component_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner => require_component_rule.clone();
                    burner_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            let new_component = Self {
                dextr_token_address,
                admin_token_address,
                account_rewards_nft_manager,
                // claim_accounts: KeyValueStore::new(),
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
                "dapp_definition" => dapp_def_address.clone(), updatable;
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
                vec![GlobalAddress::from(component_address.clone()), account_rewards_nft_manager.address().into()],
            );
            dapp_def_account.set_owner_role(rule!(require(admin_token_address.clone())));

            new_component
        }

        pub fn add_account_rewards(
            &mut self,
            reward_name: String,
            reward_token: ResourceAddress,
            account_rewards: Vec<(ComponentAddress,Decimal)>,
            rewards_bucket: Bucket,
        ) -> Bucket {
            self.add_rewards(reward_name, reward_token, account_rewards, String::from(""), rewards_bucket)
        }

         pub fn add_orders_rewards(
            &mut self,
            reward_token: ResourceAddress,
            orders_rewards_string: String,
            rewards_bucket: Bucket,
        ) -> Bucket {
            self.add_rewards(String::from(""), reward_token, vec![], orders_rewards_string, rewards_bucket)
        }

        pub fn add_rewards(
            &mut self,
            reward_name: String,
            reward_token: ResourceAddress,
            account_rewards: Vec<(ComponentAddress,Decimal)>,
            orders_rewards_string: String,
            mut rewards_bucket: Bucket,
        ) -> Bucket {
            assert!(reward_token == rewards_bucket.resource_address(), "Reward Token address must match tokens in Rewards Bucket.");
            // comment below out for production
            let _rewards_bucket_address_string =
                self.create_resource_address_string(
                    &rewards_bucket.resource_address(),
                );
            info!(
                "Reward bucket for resource {}: Amount: {}",
                _rewards_bucket_address_string,
                rewards_bucket.amount()
            );
            // comment above out for production
            let mut reward_tokens_total = Decimal::ZERO;
            if account_rewards.len() > 0 {
                reward_tokens_total = reward_tokens_total + self.load_account_rewards(reward_name.clone(), reward_token, account_rewards, true);
            }
            if orders_rewards_string != "" {
                let order_rewards = self.parse_orders_rewards_data(orders_rewards_string);
                reward_tokens_total = reward_tokens_total + self.load_orders_rewards(order_rewards, true);
            }
            if reward_tokens_total > rewards_bucket.amount() {
                panic!("Not enough tokens sent in rewards bucket. Needed {:?}, but found only {:?}.", reward_tokens_total.clone(), rewards_bucket.amount());
            }
            if self
                .claim_vaults
                .get(&rewards_bucket.resource_address())
                .is_some()
            {
                let mut claim_vault = self
                    .claim_vaults
                    .get_mut(&rewards_bucket.resource_address())
                    .unwrap();
                claim_vault.put(rewards_bucket.take(reward_tokens_total));
            } else {
                let new_vault = Vault::with_bucket(rewards_bucket.take(reward_tokens_total));
                self.claim_vaults
                    .insert(rewards_bucket.resource_address(), new_vault);
            }
            rewards_bucket
        }

        pub fn remove_account_rewards(
            &mut self, 
            reward_name: String,
            reward_token: ResourceAddress,
            account_rewards: Vec<(ComponentAddress,Decimal)>
        ) -> Bucket {
            self.remove_rewards(reward_name, reward_token, account_rewards, String::from(""))
        }
        
        pub fn remove_orders_rewards(
            &mut self, 
            reward_token: ResourceAddress,
            orders_rewards_string: String
        ) -> Bucket {
            self.remove_rewards(String::from(""), reward_token, vec![], orders_rewards_string)
        }
        
        pub fn remove_rewards(
            &mut self, 
            reward_name: String,
            reward_token: ResourceAddress,
            account_rewards: Vec<(ComponentAddress,Decimal)>,
            orders_rewards_string: String
        ) -> Bucket {
            let mut reward_tokens_removed = Decimal::ZERO;
            if account_rewards.len() > 0 {
                reward_tokens_removed = reward_tokens_removed + self.load_account_rewards(reward_name.clone(), reward_token, account_rewards, false);
            }
            if orders_rewards_string.len() > 0 {
                let order_rewards = self.parse_orders_rewards_data(orders_rewards_string);
                reward_tokens_removed = reward_tokens_removed + self.load_orders_rewards(order_rewards, false);
            }
            let mut return_bucket = Bucket::new(reward_token.clone());
            if reward_tokens_removed > Decimal::ZERO {
                let mut token_vault = self.claim_vaults.get_mut(&reward_token).expect(&format!(
                    "Could not find token vault for token {:?} to claim removed tokens.",
                    reward_token.clone()
                ));
                if token_vault.amount() >= reward_tokens_removed {
                    return_bucket.put(token_vault.take(reward_tokens_removed));
                } else {
                    panic!("Not enough tokens in claim vault for token {:?}. Required {:?}, but only found {:?}", reward_token.clone(), reward_tokens_removed.clone(), token_vault.amount());
                }
            }
            return_bucket
        }

        pub fn claim_rewards(
            &mut self,
            reward_nft_proofs: Vec<NonFungibleProof>,
            orders_proofs: Vec<NonFungibleProof>,
        ) -> Vec<Bucket> {
            info!("Starting to claim rewards!");
            let mut token_totals: HashMap<ResourceAddress, Decimal> = HashMap::new();
            let mut return_buckets: Vec<Bucket> = vec![];
            let rewards_nft_address = self.account_rewards_nft_manager.address();
            for reward_proof in reward_nft_proofs {
                assert!(reward_proof.resource_address() == rewards_nft_address.clone(), "Wrong NFT submitted. Only Dexter Claim NFTs can be submitted for claims.");
                let nfts = reward_proof.skip_checking().non_fungibles::<AccountRewardsData>();
                for nft in nfts {
                    let nft_data = nft
                        .data();
                    info!("Claim NFT Data: {:?}", nft_data);
                    for reward_name_tokens in nft_data.rewards.into_values() {
                        for (token_address_string, token_reward) in reward_name_tokens {
                            let existing_token_total = token_totals.entry(token_address_string.clone()).or_insert(Decimal::ZERO).to_owned();
                            token_totals.insert(
                                token_address_string.clone(), 
                                existing_token_total.checked_add(token_reward).expect(&format!("Could not add token reward {:?} to existing token total {:?}.", token_reward, existing_token_total))
                            );
                            info!("Token totals: {:?}", token_totals);
                        }
                    };
                    self.account_rewards_nft_manager.update_non_fungible_data::<HashMap<String, HashMap<String, Decimal>>>(nft.local_id(), "rewards", HashMap::new());
                }
                // let nft = reward_proof
                //     .check(rewards_nft_address.clone())
                //     .as_non_fungible();
                // //TODO Change this to also handle multiple ids in same proof like for orders below
                // let nft_id = nft.non_fungible_local_id();
                // let nft_data = nft
                //     .non_fungible::<AccountRewardsData>()
                //     .data();
                // for reward_name_tokens in nft_data.rewards.into_values() {
                //     for (token_address_string, token_reward) in reward_name_tokens {
                //         let existing_token_total = token_totals.entry(token_address_string.clone()).or_insert(Decimal::ZERO).to_owned();
                //         token_totals.insert(
                //             token_address_string.clone(), 
                //             existing_token_total.checked_add(token_reward).expect(&format!("Could not add token reward {:?} to existing token total {:?}.", token_reward, existing_token_total))
                //         );
                //     }
                // };
                // self.account_rewards_nft_manager.update_non_fungible_data::<HashMap<String, HashMap<String, Decimal>>>(&nft_id, "rewards", HashMap::new());
            }
            info!("Handled accounts claims");

            info!("Starting to handle order claims");
            let mut dextr_token_total = token_totals.entry(self.dextr_token_address.clone()).or_insert(Decimal::ZERO).clone();
            let mut orders_to_remove: Vec<String> = vec![];
            for orders_proof in orders_proofs {
                let proof_resource_address = orders_proof.resource_address();
                let resource_string = self.create_resource_address_string(
                            &proof_resource_address,
                        );
                let order_ids = orders_proof.skip_checking().non_fungible_local_ids();
                for order_id in order_ids {
                    let mut order_index_string =
                        resource_string.clone();
                    let order_id_string = order_id.to_string();
                    info!("order_id string: {:?}", order_id_string);
                    order_index_string.push_str(&order_id_string);
                    info!("Order_index_string {:?}", order_index_string);
                    if let Some(order_claim_amount) = self.claim_orders.get(&order_index_string)
                    {
                        dextr_token_total = dextr_token_total
                            .checked_add(order_claim_amount.clone())
                            .expect(&format!("Could not add order claim amount {:?} to total {:?}", order_claim_amount.clone(), dextr_token_total.clone()));
                        orders_to_remove.push(order_index_string.clone());
                    }
                }
            }
            token_totals.insert(
                self.dextr_token_address.clone(), 
                dextr_token_total
            );
            for order in orders_to_remove {
                self.claim_orders.remove(&order);
            }
            info!("Handled orders claims");
            for (token_address, token_reward) in token_totals {
                if self.claim_vaults.get(&token_address).is_some() {
                    let mut token_vault = self.claim_vaults.get_mut(&token_address).unwrap();
                    assert!(token_vault.amount() >= token_reward, "Not enough tokens in component to pay for claimed rewards.");
                    return_buckets.push(token_vault.take(token_reward));
                }
            }
            return_buckets
        }

        fn load_account_rewards(&mut self, reward_name: String, reward_token: ResourceAddress, account_rewards: Vec<(ComponentAddress,Decimal)>, add: bool) -> Decimal {
            let mut total_token_change = Decimal::ZERO;
            for (account_address, account_reward) in account_rewards {
                let mut skip_account = false;
                let account_id = NonFungibleLocalId::string(self.create_account_id(&account_address)).expect(&format!("Could not convert {:?} into a valid NFT ID", account_address));
                info!("Account NFT id: {:?}", account_id);
                let existing_account_data: AccountRewardsData;
                if self.account_rewards_nft_manager.non_fungible_exists(&account_id) {
                    existing_account_data = self.account_rewards_nft_manager.get_non_fungible_data(&account_id)
                } else {
                    // account_nft_exists = false;
                    existing_account_data = AccountRewardsData {
                        account_address: self.create_component_address_string(&account_address),
                        rewards: HashMap::new()
                    };
                    if add {
                        let new_nft = self.account_rewards_nft_manager.mint_non_fungible(&account_id, existing_account_data.clone()).as_non_fungible();
                        let account_component: Global<AnyComponent> = Global::from(account_address);
                        let returned_bucket: Option<NonFungibleBucket> = account_component.call::<(NonFungibleBucket,Option<ResourceOrNonFungible>),_>("try_deposit_or_refund", &(new_nft, None));
                        if let Some(returned_nft) = returned_bucket {
                            info!("Could not deposit nft to account {:?}", account_address);
                            returned_nft.burn();
                            skip_account = true;
                        };
                        info!("Account received NFT");
                    } else {
                        skip_account = true;
                    }
                }
                info!("Existing account data: {:?}", existing_account_data);
                let mut existing_account_rewards = existing_account_data.rewards;
                if !skip_account {
                    let mut existing_name_data = existing_account_rewards
                        .entry(reward_name.clone())
                        .or_insert(HashMap::new())
                        .clone();
                    info!("Existing name data: {:?}", existing_name_data);
                    let mut existing_token_total = existing_name_data
                        .entry(reward_token.clone())
                        .or_insert(Decimal::ZERO)
                        .clone();
                    let mut token_change = account_reward.clone();
                    if add {
                        existing_token_total =
                            existing_token_total.checked_add(token_change).expect(
                                "Could not add new token reward to existing token total",
                            );
                    } else {
                        token_change =
                            existing_token_total.min(account_reward.clone());
                        existing_token_total =
                            existing_token_total.checked_sub(token_change).expect(
                                "Could not remove token reward from existing token total",
                            );
                    }
                    if existing_token_total > Decimal::ZERO {
                        existing_name_data
                            .insert(reward_token.clone(), existing_token_total.to_owned());
                    } else {
                        existing_name_data.remove(&reward_token);
                    }
                    info!("Existing name data (after update) {:?}", existing_name_data);
                    total_token_change = total_token_change + token_change;
                    info!("Total token reward: {:?}", total_token_change);
                    if existing_name_data.len() > 0 {
                        existing_account_rewards
                            .insert(reward_name.clone(), existing_name_data.to_owned());
                    } else {
                        existing_account_rewards.remove(&reward_name);
                    }
                    info!("Existing account rewards: {:?}", existing_account_rewards);
                    self.account_rewards_nft_manager.update_non_fungible_data::<HashMap<String, HashMap<ResourceAddress, Decimal>>>(&account_id, "rewards", existing_account_rewards);
                }
            }
            total_token_change
        }

        fn load_orders_rewards(
            &mut self,
            orders_data: Vec<JsonPairOrderRewards>,
            add: bool,
        ) -> Decimal {
            let mut total_orders_reward_amount = Decimal::ZERO;
            for pair_order_rewards_data in &orders_data {
                let pair_address_string = pair_order_rewards_data.pair_receipt_address.clone();
                for (order_id, order_reward_amount) in &pair_order_rewards_data.pair_rewards {
                    let mut order_id_string = pair_address_string.clone();
                    order_id_string.push_str("#");
                    order_id_string.push_str(&order_id.to_string());
                    order_id_string.push_str("#");
                    info!("Order id string: {:?}", order_id_string);
                    total_orders_reward_amount = total_orders_reward_amount
                        .checked_add(order_reward_amount.clone())
                        .expect("Could not add to total_orders_reward_amount.");
                    if add {
                        self.claim_orders
                            .insert(order_id_string, order_reward_amount.clone());
                    } else {
                        self.claim_orders.remove(&order_id_string);
                    }
                }
            }
            total_orders_reward_amount
        }

        fn parse_orders_rewards_data(&self, orders_rewards_data_str: String) -> Vec<JsonPairOrderRewards> {
            let mut result = vec![];
            let changed_rewards_data_str = orders_rewards_data_str.replace("'", "\"");
            let extracted_data =
                json::parse(&changed_rewards_data_str).expect("Invalid JSON specified!");
            if let JsonValue::Array(orders_data) = extracted_data {
                for pair_data_obj in orders_data {
                    if let JsonValue::Object(pair_data) = pair_data_obj {
                        // info!("Pair data: {:?}", pair_data);
                        let mut pair_receipt_address = String::from("");
                        let mut pair_rewards: Vec<(u64, Decimal)> = vec![];
                        for (field_key, field_value) in pair_data.iter() {
                            // info!(
                            //     "Pair data field key: {:?} => value {:?}",
                            //     field_key, field_value
                            // );
                            match field_key {
                                "pair_receipt_address" => {
                                    pair_receipt_address = self
                                        .get_string_value(field_value, field_key);
                                }
                                "pair_rewards" => {
                                    if let JsonValue::Array(pair_order_rewards) =
                                        field_value
                                    {
                                        // info!(
                                        //     "Pair rewards: {:?}",
                                        //     pair_order_rewards
                                        // );
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
                        // info!("Loaded pair rewards: {:?}", pair_rewards);
                        if pair_receipt_address == "" {
                            panic!("Found pair rewards without preceding pair receipt address.");
                        } else {
                            result.push(JsonPairOrderRewards {
                                pair_receipt_address,
                                pair_rewards,
                            })
                        }
                    } else {
                        panic!("Pair Orders data must be an object.");
                    }
                }
            } else {
                panic!("Orders rewards data must be an array.")
            }
            result
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
            // info!("Get number value for {:?}", json_value);
            match json_value {
                JsonValue::Number(field_value) => {
                    // info!("{}: {:?}", field_name, field_value);
                    let field_number = field_value
                        .as_fixed_point_u64(0)
                        .expect(&format!("Could not convert {} to a u64", field_value));
                    // info!("Field number: {}", field_number);
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

        fn create_resource_address_string(&self, address: &ResourceAddress) -> String {
            if self.env == "mainnet" || self.env == "stokenet" {
                Runtime::bech32_encode_address(address.clone())
            } else {
                address.to_hex()
            }
        }

        fn create_component_address_string(&self, address: &ComponentAddress) -> String {
            if self.env == "mainnet" || self.env == "stokenet" {
                Runtime::bech32_encode_address(address.clone())
            } else {
                address.to_hex()
            }
        }

        fn create_account_id(&self, address: &ComponentAddress) -> String {
            if self.env == "mainnet" || self.env == "stokenet" {
                let full_address = Runtime::bech32_encode_address(address.clone());
                let address_split: Vec<&str> = full_address.split("1").collect();
                let mut id = String::from("");
                if address_split.len() > 0 {
                    id = address_split[1].to_string();
                }
                id
            } else {
                address.to_hex()
            }
        }
    }
}
