/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_CHC_ICON: &str = "data:image/svg+xml,%3Csvg width='5e3' height='3e3' version='1.1' viewBox='0 0 5e3 3e3' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='m1e3 2e3c-402-13-755-381-745-784-2-3%0A42 241-673 576-757 270-75 577 7 771 210 92 58 68 163 15 237-119 270-270 525-394 793 137-319 333-611 452-938-225-334-728-432-1e3 -205-301 192-443 6%0A03-307 936 117 309 441 532 774 505 211-12 424-108 549-283 63-134 117-273 177-409-84 156-127 333-227 480-153 147-369 226-580 214zm-43-231c-85 20-6-%0A170-41-58-5 107 159 55 120-34 9-34-13 165-79 92zm264-61c74 11 184 70 233-18 45-120 109-232 162-349 20-41 39-80 8-15-67 141-137 280-202 421-69-0.9-%0A133-32-201-40zm-443 14c-87 5 2-147-100-135-90-55-154-153-162-259 15 126 101 245 223 287-44 81 83 180 105 61 10-34-17 114-65 46zm137-67c-13-7 20 5 0 0zm-48-8c33 4 7 0.9 0 0zm191 3c31-9 19-2 0 0zm48-22c4-17 0.01 13 0 0zm0.7-12c9-11-11 44 0 0zm-160-141c14 4 22 7 0 0zm173-5c-0.8 45-8 54-0.3 3zm-%0A143 5-18-0.6zm104-3c-9 1-6 0.7 0 0zm-270-37c59 18 72 22 0 0zm-29-19c7-39-63-63-9-20 19 13 40 46 9 20zm1e3 -108c-4-32-68-56-9-32 7 3 18 79 9 32zm-1%0Ae3 2c12 40 6 51 0 0zm-0.9-3c2-7 0.6 10 0 0zm-221-3c-2-11 3 4 0 0zm-0.02-9 0.04 5zm221-5c0.6 31 8 25 0 0zm-222 0.3c-110-161-219-321-329-482 110 161 219 321 329 482zm225-51c-4 19-1 58 0.08 16zm-225 45c4-36 6-64 1-11zm221-9c5-23 0.3-32 0 0zm985-29c65 18-33-5 0 0zm-1e3 -59c-8 53-5 50 0 0zm6-29 1%0A-0.5-1 0.5zm1-3 1-0.4zm0.8-2c42-153 150-276 268-377 92-27 189-64 226-153 50-22 101 102 53 18-59-42-103 35-117 72-92 43-199 66-261 153-77 82-131 18%0A2-170 287zm364-201c25-29 136-63 51-31-73 38-116 112-164 176 30-54 70-102 113-145zm848 137c50-104 109-203 154-309-55 101-103 206-154 309zm-0.9 2c21%0A-25 98-124 81-101-27 34-54 67-81 101zm-410-108c-19-29-137-77-109-67 39 18 77 39 109 67zm8 3c23-31 100-136 91-117-29 40-59 79-91 117zm-165-87c-54-4%0A-52-14 0 0zm-148-7c20-20 14 7 0 0zm21-4c33-1 20 4 0 0zm399-48c-59-60-165-70-189-160-34-16-127 19-43-10 74 30 108 116 190 139 14 10 28 20 42 30zm-2%0A90-130c-40-5 9-3 3-1zm-21-4c9 9-24-9 0 0zm-31-1c2-7 13 12 0 0zm-12-1c5-4 8 12 0 0z' stroke-width='.8'/%3E%3C/svg%3E";


#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "ChaveroCoin Token".to_string(),
                symbol: "ChC".to_string(),
                icon: Some(DATA_IMAGE_SVG_CHC_ICON.to_string()),
                reference: Some("https://chaverocoin.com/token.json".to_string()),
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), total_supply.into());
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
