#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::collections::HashMap as StorageHashMap;
    #[ink(storage)]
    pub struct Erc20 {
        issuer: AccountId,
        total_supply: Balance,
        balances: StorageHashMap<AccountId, Balance>,
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Create {
        #[ink(topic)]
        from: AccountId,
        total_supply: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct TransferFrom {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        from: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Issue {
        #[ink(topic)]
        issuer: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficentBalance,
        InsufficentAllowance,
        NotIssuer,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, total_supply);
            let instance = Self {
                issuer: caller,
                total_supply: total_supply,
                balances: balances,
                allowances: StorageHashMap::new(),
            };

            Self::env().emit_event(Create {
                from: caller,
                total_supply: total_supply,
            });

            instance
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();

            self.approve_help(who, spender, value)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            *self.allowances.get(&(owner, spender)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();

            self.transfer_help(who, to, value)
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let who = Self::env().caller();

            self.transfer_from_help(who, from, to, value)
        }

        #[ink(message)]
        pub fn burn(&mut self, value: Balance) -> Result<()> {
            let who = Self::env().caller();

            self.burn_help(who, value)
        }

        #[ink(message)]
        pub fn issue(&mut self, value: Balance) -> Result<()> {
            let who = Self::env().caller();

            self.issue_help(who, value)
        }

        pub fn transfer_help(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of(from);

            if from_balance < value {
                return Err(Error::InsufficentBalance);
            }

            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            Self::env().emit_event(Transfer {
                from: from,
                to: to,
                value: value,
            });

            Ok(())
        }

        pub fn approve_help(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            value: Balance,
        ) -> Result<()> {
            let owner_balance = self.balance_of(owner);

            if owner_balance < value {
                return Err(Error::InsufficentBalance);
            }

            self.balances.insert(owner, owner_balance - value);
            let allowance = self.allowance(owner, spender);
            self.allowances.insert((owner, spender), allowance + value);

            Self::env().emit_event(Approval {
                owner: owner,
                spender: spender,
                value: value,
            });

            Ok(())
        }

        pub fn transfer_from_help(
            &mut self,
            from: AccountId,
            owner: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let allowance = self.allowance(owner, from);

            if allowance < value {
                return Err(Error::InsufficentAllowance);
            }

            self.allowances.insert((owner, from), allowance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            Self::env().emit_event(TransferFrom {
                from: from,
                owner: owner,
                to: to,
                value: value,
            });

            Ok(())
        }

        pub fn burn_help(&mut self, from: AccountId, value: Balance) -> Result<()> {
            let from_balance = self.balance_of(from);

            if from_balance < value {
                return Err(Error::InsufficentBalance);
            }

            self.balances.insert(from, from_balance - value);
            self.total_supply = self.total_supply() - value;

            Self::env().emit_event(Burn {
                from: from,
                value: value,
            });

            Ok(())
        }

        pub fn issue_help(&mut self, from: AccountId, value: Balance) -> Result<()> {
            if from != self.issuer {
                return Err(Error::NotIssuer);
            }

            let from_balance = self.balance_of(from);

            self.balances.insert(from, from_balance + value);
            self.total_supply = self.total_supply() + value;

            Self::env().emit_event(Issue {
                issuer: from,
                value: value,
            });

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn create_contract_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(ink_env::test::recorded_events().count(), 1);
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);

            assert_eq!(erc20.transfer(accounts.bob, 100), Ok(()));

            assert_eq!(erc20.balance_of(accounts.alice), 900);
            assert_eq!(erc20.balance_of(accounts.bob), 100);
            assert_eq!(ink_env::test::recorded_events().count(), 2);
        }

        #[ink::test]
        fn transfer_failed_with_insufficentbalance() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);

            assert_eq!(
                erc20.transfer(accounts.bob, 2000),
                Err(Error::InsufficentBalance)
            );
        }

        #[ink::test]
        fn approve_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.charlie), 0);
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 0);

            assert_eq!(erc20.approve(accounts.bob, 100), Ok(()));

            assert_eq!(erc20.balance_of(accounts.alice), 900);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.charlie), 0);
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 100);
            assert_eq!(ink_env::test::recorded_events().count(), 2);

            let callee =
                ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.charlie, 50),
                Ok(())
            );

            assert_eq!(erc20.balance_of(accounts.alice), 900);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.charlie), 50);
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 50);
        }

        #[ink::test]
        fn approve_failed_with_insufficientallowance() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.charlie), 0);
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 0);

            assert_eq!(erc20.approve(accounts.bob, 100), Ok(()));

            assert_eq!(erc20.balance_of(accounts.alice), 900);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.charlie), 0);
            assert_eq!(erc20.allowance(accounts.alice, accounts.bob), 100);
            assert_eq!(ink_env::test::recorded_events().count(), 2);

            let callee =
                ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            assert_eq!(
                erc20.transfer_from(accounts.alice, accounts.charlie, 500),
                Err(Error::InsufficentAllowance)
            );
        }

        #[ink::test]
        fn burn_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);

            assert_eq!(erc20.burn(100), Ok(()));

            assert_eq!(erc20.balance_of(accounts.alice), 900);
            assert_eq!(ink_env::test::recorded_events().count(), 2);
        }

        #[ink::test]
        fn burn_failed_with_insufficientbalance() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);

            assert_eq!(erc20.burn(2000), Err(Error::InsufficentBalance));
        }

        #[ink::test]
        fn issue_works() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);

            assert_eq!(erc20.issue(1000), Ok(()));

            assert_eq!(erc20.total_supply(), 2000);
            assert_eq!(erc20.balance_of(accounts.alice), 2000);
            assert_eq!(ink_env::test::recorded_events().count(), 2);
        }

        #[ink::test]
        fn issue_failed_with_notissuer() {
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

            let mut erc20 = Erc20::new(1000);

            assert_eq!(erc20.total_supply(), 1000);
            assert_eq!(erc20.balance_of(accounts.alice), 1000);

            let callee =
                ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4]));
            data.push_arg(&accounts.bob);
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            assert_eq!(erc20.issue(1000), Err(Error::NotIssuer));
        }
    }
}
