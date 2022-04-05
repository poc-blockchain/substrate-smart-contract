#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod fundraiser {
    use ink_storage::{
        traits:: {
            SpreadAllocate,
            PackedLayout,
            SpreadLayout,
        },
        Mapping,
    };
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;

    type FundingId = u32;
    const WRONG_TRANSACTION_ID: &str =
        "The user specified an invalid transaction id. Abort.";

    /// Errors that can occur upon calling this contract.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the call failed.
        TransactionFailed,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Fundraiser {
        // Stores a single `bool` value on the storage.
        name: String,
        owner: AccountId,
        transaction_list: Transactions,
        /// Map the transaction id to its unexecuted transaction.
        transactions: Mapping<FundingId, Transaction>,
        current_funding: Mapping<FundingId, Balance>,
    }

    #[derive(SpreadLayout, SpreadAllocate, Default)]
    #[cfg_attr(
        feature = "std",
        derive(
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Transactions {
        /// Just store all transaction ids packed.
        transactions: Vec<FundingId>,
        /// We just increment this whenever a new transaction is created.
        /// We never decrement or defragment. For now, the contract becomes defunct
        /// when the ids are exhausted.
        next_id: FundingId
    }

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(scale::Encode, scale::Decode, SpreadLayout, SpreadAllocate, PackedLayout)]
    #[cfg_attr(
        feature = "std",
        derive(
            scale_info::TypeInfo,
            ink_storage::traits::StorageLayout
        )
    )]
    pub struct Transaction {
        /// The `AccountId` of the contract that is called in this transaction.
        pub callee: AccountId,
        /// The amount of chain balance that is transferred to the callee.
        pub expected_value: Balance,
    }

    impl Fundraiser {
        //// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(name_value: String, owner_value: AccountId) -> Self {
            // This call is required in order to correctly initialize the
            // `Mapping`s of our contract.
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.name = name_value;
                contract.owner = owner_value;
                contract.transactions = Default::default();
            })
        }
        
        /// Add a new transaction candidate to the contract.
        ///
        /// This also confirms the transaction for the caller. This can be called by any owner.
        #[ink(message)]
        pub fn create_a_funding(
            &mut self,
            expected_value: Balance,
        ) -> FundingId {
            
            // Generate transaction id for the next submit request
            let fund_id = self.transaction_list.next_id;
            self.transaction_list.next_id = fund_id.checked_add(1).expect("Transaction ids exhausted.");
            
            // Create new transaction object
            let transaction = Transaction {
                callee: self.env().caller(),
                expected_value: expected_value,
            };
            self.transactions.insert(fund_id, &transaction);
            self.transaction_list.transactions.push(fund_id);

            fund_id
        }

        /// Read transaction by its id
        #[ink(message)]
        pub fn get_funding(&self, fund_id: FundingId) -> Option<Transaction> {
            let transaction = self.transactions.get(&fund_id);
            transaction
        }

        /// Read transaction current funding
        #[ink(message)]
        pub fn get_funding_status(&self, fund_id: FundingId) -> Option<Balance> {
            self.current_funding.get(&fund_id)
        }

        #[ink(message, payable)]
        pub fn fund(&mut self, fund_id: FundingId) {
            let caller = self.env().caller();
            let value = self.env().transferred_value();

            self.ensure_transaction_exists(fund_id);
            self.ensure_funding_is_not_exceed(fund_id);
            let funding = self.current_funding.get(fund_id);
            let mut f = value;
            if funding.is_some() {
                f = funding.unwrap().checked_add(value).expect("Funding exhausted.");
            }

            self.current_funding.remove(fund_id);
            self.current_funding.insert(fund_id, &f);
            
            ink_env::debug_println!("thanks for the funding of {:?} from {:?}", value, caller);
            ink_env::debug_println!("transaction id {:?} current balance {:?}", fund_id, value);
        }

        #[ink(message)]
        pub fn withdraw(&mut self, fund_id: FundingId) {
            self.ensure_transaction_exists(fund_id);
            self.ensure_transaction_owner(fund_id, self.env().caller());
            self.ensure_funding_is_full(fund_id);

            let funding = self.current_funding.get(fund_id).unwrap();
            
            ink_env::debug_println!("contract balance: {}", self.env().balance());

            assert!(funding <= self.env().balance(), "insufficient funds!");

            if self.env().transfer(self.env().caller(), funding).is_err() {
                ink_env::debug_println!("Failed");
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
            self.transactions.remove(&fund_id);
        }

        #[ink(message)]
        pub fn current_balance(&self) -> Balance {
            self.env().balance()
        }

        #[ink(message)]
        pub fn print(&self) {
            let caller = self.env().caller();
            ink_env::debug_println!("got a call from {:?}", caller);
        }

        /// Panic if the transaction `fund_id` does not exit.
        fn ensure_transaction_exists(&self, fund_id: FundingId) {
            self.transactions.get(fund_id).expect(WRONG_TRANSACTION_ID);
        }

        /// Panic if the transaction `fund_id` does not exit.
        fn ensure_transaction_owner(&self, fund_id: FundingId, caller_id: AccountId) {
            let transaction = self.transactions.get(fund_id).unwrap();
            assert_eq!(transaction.callee, caller_id);
        }

        /// Panic if the current funding is matched the expected
        fn ensure_funding_is_not_exceed(&self, fund_id: FundingId) {
            let transaction = self.transactions.get(fund_id).unwrap();
            let current_funding = self.current_funding.get(fund_id);
            if current_funding.is_some() {
                assert!(current_funding.unwrap() < transaction.expected_value);
            }
        }

        /// Panic if the current funding is full
        fn ensure_funding_is_full(&self, fund_id: FundingId) {
            let transaction = self.transactions.get(fund_id).unwrap();
            let current_funding = self.current_funding.get(fund_id).unwrap();
            assert!(current_funding >= transaction.expected_value);
        }
    }
}
