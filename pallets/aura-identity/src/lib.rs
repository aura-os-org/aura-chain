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
    pub type AuraIdentities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        AuraIdRecord,
    >;

    #[pallet::storage]
    pub type DidIndex<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        [u8; 32],
        T::AccountId,
    >;

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
                created: frame_system::Pallet::<T>::block_number(),
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
            trustees: Vec<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: Implement social recovery logic
            Self::deposit_event(Event::RecoverySetup { account: who, shares });
            
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn generate_did(public_key: &[u8; 32]) -> [u8; 32] {
            sp_io::hashing::blake2_256(public_key)
        }
    }

    #[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct AuraIdRecord {
        pub did: [u8; 32],
        pub public_key: [u8; 32],
        pub recovery_config: Vec<u8>,
        pub created: T::BlockNumber,
    }
}
