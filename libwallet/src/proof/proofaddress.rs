// Copyright 2020 The MWC Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::base58::Base58;
use crate::address;
use crate::error::Error;
use crate::grin_util::secp::key::PublicKey;
use crate::proof::crypto;
use grin_core::global;
use grin_wallet_util::grin_keychain::{Identifier, Keychain};
use serde::{Deserialize, Deserializer, Serializer};
use std::fmt::{self, Display};

/// Address prefixes for mainnet
pub const PROOFABLE_ADDRESS_VERSION_MAINNET: [u8; 2] = [1, 69];
/// Address prefixes for floonet
pub const PROOFABLE_ADDRESS_VERSION_TESTNET: [u8; 2] = [1, 121];

/// Address that can have a proof. Such address need to be able to convertable to
/// the public key
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvableAddress {
	/// Public key that is an address
	pub public_key: String,
	/// Place holder for mwc713 backcompability. Value is empty string
	pub domain: String,
	/// /// Place holder for mwc713 backcompability. Value is None
	pub port: Option<u16>,
}

impl Display for ProvableAddress {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.public_key)
	}
}

impl ProvableAddress {
	/// new instance
	pub fn from_str(public_key: &str) -> Result<Self, Error> {
		// Just check if it works
		PublicKey::from_base58_check(public_key, version_bytes())?;

		Ok(Self {
			public_key: String::from(public_key),
			domain: String::new(),
			port: None,
		})
	}

	/// Create address from public key
	pub fn from_pub_key(public_key: &PublicKey) -> Self {
		Self {
			public_key: public_key.to_base58_check(version_bytes()),
			domain: String::new(),
			port: None,
		}
	}

	/// Get public key that represent this address
	pub fn public_key(&self) -> Result<PublicKey, Error> {
		PublicKey::from_base58_check(&self.public_key, version_bytes())
	}
}

/// provable address prefix.
pub fn version_bytes() -> Vec<u8> {
	if global::is_mainnet() {
		PROOFABLE_ADDRESS_VERSION_MAINNET.to_vec()
	} else {
		PROOFABLE_ADDRESS_VERSION_TESTNET.to_vec()
	}
}

/// provable address public key
pub fn payment_proof_address<K>(
	keychain: &K,
	parent_key_id: &Identifier,
	index: u32,
) -> Result<ProvableAddress, Error>
where
	K: Keychain,
{
	let sender_address_secret_key =
		address::address_from_derivation_path(keychain, &parent_key_id, index)?;
	let sender_address_pub_key = crypto::public_key_from_secret_key(&sender_address_secret_key)?;
	Ok(ProvableAddress::from_pub_key(&sender_address_pub_key))
}

///
pub fn payment_proof_address_pubkey<K>(
	keychain: &K,
	parent_key_id: &Identifier,
	index: u32,
) -> Result<PublicKey, Error>
where
	K: Keychain,
{
	let sender_address_secret_key =
		address::address_from_derivation_path(keychain, &parent_key_id, index)?;
	crypto::public_key_from_secret_key(&sender_address_secret_key)
}

/// ProvableAddress
pub fn proof_address_from_string<'de, D>(deserializer: D) -> Result<ProvableAddress, D::Error>
where
	D: Deserializer<'de>,
{
	use serde::de::Error;

	String::deserialize(deserializer).and_then(|string| {
		ProvableAddress::from_str(&string).map_err(|err| {
			Error::custom(format!(
				"Fail to parse provable address {}, {}",
				string, err
			))
		})
	})
}

/// Seralizes a provableAddress.
pub fn as_string<S>(address: &ProvableAddress, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	serializer.serialize_str(&address.public_key)
}

/// ProvableAddress
pub fn option_proof_address_from_string<'de, D>(
	deserializer: D,
) -> Result<Option<ProvableAddress>, D::Error>
	where
		D: Deserializer<'de>,
{
	use serde::de::Error;

	Option::<String>::deserialize(deserializer).and_then(|res| match res {
		Some(string) => ProvableAddress::from_str(&string)
			.map_err(|err| {
				Error::custom(format!(
					"Fail to parse provable address {}, {}",
					string, err
				))
			})
			.and_then(|address: ProvableAddress| {
				return Ok(Some(address));
			}),
		None => Ok(None),
	})
}

/// Seralizes a provableAddress.
pub fn option_as_string<S>(
	address: &Option<ProvableAddress>,
	serializer: S,
) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
{
	match address {
		Some(address) => serializer.serialize_str(&address.public_key),
		None => serializer.serialize_none(),
	}
}
