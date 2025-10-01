#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_aura_id)]
	pub type AuraIdentities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, AuraIdRecord>;

	#[pallet::storage]
	pub type DidIndex<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AuraIdCreated { account: T::AccountId, did: [u8; 32] },
		RecoverySetup { account: T::AccountId, shares: u8 },
	}

	#[pallet::error]
	pub enum Error<T> {
		AuraIdAlreadyExists,
		AuraIdNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn create_aura_id(
			origin: OriginFor<T>,
			public_key: [u8; 32],
			recovery_config: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!AuraIdentities::<T>::contains_key(&who), Error::<T>::AuraIdAlreadyExists);

			let did = Self::generate_did(&public_key);

			let record = AuraIdRecord {
				did,
				public_key,
				recovery_config,
				created: <T as frame_system::Config>::BlockNumber::from(0u32), // Temporary fix
			};

			AuraIdentities::<T>::insert(&who, record.clone());
			DidIndex::<T>::insert(did, &who);

			Self::deposit_event(Event::AuraIdCreated { account: who, did });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn setup_recovery(
			origin: OriginFor<T>,
			shares: u8,
			_trustees: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// TODO: Implement social recovery logic
			Self::deposit_event(Event::RecoverySetup { account: who, shares });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn generate_did(public_key: &[u8; 32]) -> [u8; 32] {
			sp_io::hashing::blake2_256(public_key)
		}
	}

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AuraIdRecord {
		pub did: [u8; 32],
		pub public_key: [u8; 32],
		pub recovery_config: BoundedVec<u8, ConstU32<1024>>, // Use bounded vec instead of Vec
		pub created: u32, // Use concrete type
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{assert_noop, assert_ok, BoundedVec};
	use sp_core::H256;
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup},
		BuildStorage,
	};

	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test {
			System: frame_system,
			AuraIdentity: pallet,
		}
	);

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Nonce = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Block = Block;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = <Test as frame_system::Config>::BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl Config for Test {
		type RuntimeEvent = RuntimeEvent;
	}

	#[test]
	fn test_did_generation() {
		let public_key = [1u8; 32];
		let did = Pallet::<Test>::generate_did(&public_key);

		// DID должен быть хэшом публичного ключа
		assert_eq!(did.len(), 32);
		assert_ne!(did, public_key); // Должен отличаться от исходного ключа
	}

	#[test]
	fn test_create_aura_id() {
		new_test_ext().execute_with(|| {
			let account_id = 1;
			let public_key = [2u8; 32];
			let recovery_config: BoundedVec<u8, ConstU32<1024>> = vec![1, 2, 3].try_into().unwrap();

			// Создание Aura ID должно работать
			assert_ok!(AuraIdentity::create_aura_id(
				RuntimeOrigin::signed(account_id),
				public_key,
				recovery_config.to_vec() // Convert back to Vec for call
			));

			// Проверяем что запись создалась
			let record = AuraIdentity::get_aura_id(account_id).unwrap();
			assert_eq!(record.public_key, public_key);
			assert_eq!(record.recovery_config, recovery_config);
		});
	}

	#[test]
	fn test_duplicate_aura_id() {
		new_test_ext().execute_with(|| {
			let account_id = 1;
			let public_key = [3u8; 32];
			let recovery_config: BoundedVec<u8, ConstU32<1024>> = vec![1, 2, 3].try_into().unwrap();

			// Первое создание - ок
			assert_ok!(AuraIdentity::create_aura_id(
				RuntimeOrigin::signed(account_id),
				public_key,
				recovery_config.to_vec() // Convert back to Vec for call
			));

			// Второе создание для того же аккаунта - ошибка
			assert_noop!(
				AuraIdentity::create_aura_id(
					RuntimeOrigin::signed(account_id),
					public_key,
					recovery_config.to_vec() // Convert back to Vec for call
				),
				Error::<Test>::AuraIdAlreadyExists
			);
		});
	}

	// Вспомогательная функция для тестов
	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		t.into()
	}
}
