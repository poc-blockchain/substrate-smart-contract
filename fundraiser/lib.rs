#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod fundraiser {
    use ink_storage::{
        traits:: {
            SpreadAllocate,
            SpreadLayout,
            StorageLayout
        },
        Mapping,
    };
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;

    type TransactionId = u32;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Fundraiser {
        // Stores a single `bool` value on the storage.
        name: String,
        owner: AccountId,
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
        transactions: Vec<TransactionId>,
        /// We just increment this whenever a new transaction is created.
        /// We never decrement or defragment. For now, the contract becomes defunct
        /// when the ids are exhausted.
        next_id: TransactionId
    }

    /// A Transaction is what every `owner` can submit for confirmation by other owners.
    /// If enough owners agree it will be executed by the contract.
    #[derive(SpreadLayout, SpreadAllocate)]
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
        pub transferred_value: Balance
    }




    impl Fundraiser {
        //// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(name_value: String, owner_value: AccountId) -> Self {
            Self { 
                name: name_value,
                owner: owner_value,
            }
        }
        
        #[ink(message, payable)]
        pub fn fund(&self) {
            let caller = self.env().caller();
            let value = self.env().transferred_value();
            ink_env::debug_println!("thanks for the funding of {:?} from {:?}", value, caller);
        }

        #[ink(message)]
        pub fn withdraw(&mut self, value: Balance) {
            assert_eq!(self.env().caller(), self.owner);
            ink_env::debug_println!("requested value: {}", value);
            ink_env::debug_println!("contract balance: {}", self.env().balance());

            assert!(value <= self.env().balance(), "insufficient funds!");

            if self.env().transfer(self.env().caller(), value).is_err() {
                ink_env::debug_println!("Failed");
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
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
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let fundraiser = Fundraiser::default();
            assert_eq!(fundraiser.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut fundraiser = Fundraiser::new(false);
            assert_eq!(fundraiser.get(), false);
            fundraiser.flip();
            assert_eq!(fundraiser.get(), true);
        }
    }
}
