// Bitcoin Dev Kit
// Written in 2020 by Alekos Filini <alekos.filini@gmail.com>
//
// Copyright (c) 2020-2021 Bitcoin Dev Kit Developers
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! Address validation callbacks
//!
//! The typical usage of those callbacks is for displaying the newly-generated address on a
//! hardware wallet, so that the user can cross-check its correctness.
//!
//! More generally speaking though, these callbacks can also be used to "do something" every time
//! an address is generated, without necessarily checking or validating it.
//!
//! An address validator can be attached to a [`Wallet`](super::Wallet) by using the
//! [`Wallet::add_address_validator`](super::Wallet::add_address_validator) method, and
//! whenever a new address is generated (either explicitly by the user with
//! [`Wallet::get_address`](super::Wallet::get_address) or internally to create a change
//! address) all the attached validators will be polled, in sequence. All of them must complete
//! successfully to continue.
//!
//! ## Example
//!
//! ```
//! # use std::sync::Arc;
//! # use bitcoin::*;
//! # use bdk::address_validator::*;
//! # use bdk::database::*;
//! # use bdk::*;
//! # use bdk::wallet::AddressIndex::New;
//! #[derive(Debug)]
//! struct PrintAddressAndContinue;
//!
//! impl AddressValidator for PrintAddressAndContinue {
//!     fn validate(
//!         &self,
//!         keychain: KeychainKind,
//!         hd_keypaths: &HDKeyPaths,
//!         script: &Script
//!     ) -> Result<(), AddressValidatorError> {
//!         let address = Address::from_script(script, Network::Testnet)
//!             .as_ref()
//!             .map(Address::to_string)
//!             .unwrap_or(script.to_string());
//!         println!("New address of type {:?}: {}", keychain, address);
//!         println!("HD keypaths: {:#?}", hd_keypaths);
//!
//!         Ok(())
//!     }
//! }
//!
//! let descriptor = "wpkh(tpubD6NzVbkrYhZ4Xferm7Pz4VnjdcDPFyjVu5K4iZXQ4pVN8Cks4pHVowTBXBKRhX64pkRyJZJN5xAKj4UDNnLPb5p2sSKXhewoYx5GbTdUFWq/*)";
//! let mut wallet = Wallet::new_offline(descriptor, None, Network::Testnet, MemoryDatabase::default())?;
//! wallet.add_address_validator(Arc::new(PrintAddressAndContinue));
//!
//! let address = wallet.get_address(New)?;
//! println!("Address: {}", address);
//! # Ok::<(), bdk::Error>(())
//! ```

use std::fmt;

use bitcoin::Script;

use crate::descriptor::HDKeyPaths;
use crate::types::KeychainKind;

/// Errors that can be returned to fail the validation of an address
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressValidatorError {
    /// User rejected the address
    UserRejected,
    /// Network connection error
    ConnectionError,
    /// Network request timeout error
    TimeoutError,
    /// Invalid script
    InvalidScript,
    /// A custom error message
    Message(String),
}

impl fmt::Display for AddressValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AddressValidatorError {}

/// Trait to build address validators
///
/// All the address validators attached to a wallet with [`Wallet::add_address_validator`](super::Wallet::add_address_validator) will be polled
/// every time an address (external or internal) is generated by the wallet. Errors returned in the
/// validator will be propagated up to the original caller that triggered the address generation.
///
/// For a usage example see [this module](crate::address_validator)'s documentation.
pub trait AddressValidator: Send + Sync + fmt::Debug {
    /// Validate or inspect an address
    fn validate(
        &self,
        keychain: KeychainKind,
        hd_keypaths: &HDKeyPaths,
        script: &Script,
    ) -> Result<(), AddressValidatorError>;
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use crate::wallet::test::{get_funded_wallet, get_test_wpkh};
    use crate::wallet::AddressIndex::New;

    #[derive(Debug)]
    struct TestValidator;
    impl AddressValidator for TestValidator {
        fn validate(
            &self,
            _keychain: KeychainKind,
            _hd_keypaths: &HDKeyPaths,
            _script: &bitcoin::Script,
        ) -> Result<(), AddressValidatorError> {
            Err(AddressValidatorError::InvalidScript)
        }
    }

    #[test]
    #[should_panic(expected = "InvalidScript")]
    fn test_address_validator_external() {
        let (mut wallet, _, _) = get_funded_wallet(get_test_wpkh());
        wallet.add_address_validator(Arc::new(TestValidator));

        wallet.get_address(New).unwrap();
    }

    #[test]
    #[should_panic(expected = "InvalidScript")]
    fn test_address_validator_internal() {
        let (mut wallet, descriptors, _) = get_funded_wallet(get_test_wpkh());
        wallet.add_address_validator(Arc::new(TestValidator));

        let addr = testutils!(@external descriptors, 10);
        let mut builder = wallet.build_tx();
        builder.add_recipient(addr.script_pubkey(), 25_000);
        builder.finish().unwrap();
    }
}
