#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*, 
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
		Blake2_128Concat, BoundedVec
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use scale_info::TypeInfo;

	// Константы для Social Recovery
	pub const MAX_TRUSTEES: u32 = 10;
	pub const MIN_THRESHOLD: u8 = 2;
	pub const MAX_THRESHOLD: u8 = 10;
	pub const DEFAULT_RECOVERY_DELAY: u32 = 14400; // ~24 часа

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		
		/// Валюта для депозитов
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		
		/// Максимальное количество доверенных контактов
		#[pallet::constant]
		type MaxTrustees: Get<u32>;
		
		/// Депозит для настройки recovery
		#[pallet::constant]
		type RecoveryDeposit: Get<BalanceOf<Self>>;
	}

	/// Тип для баланса
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// ========== СУЩЕСТВУЮЩИЕ СТРУКТУРЫ ==========

	#[pallet::storage]
	#[pallet::getter(fn get_aura_id)]
	pub type AuraIdentities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, AuraIdRecord>;

	#[pallet::storage]
	pub type DidIndex<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 32], T::AccountId>;

	// ========== НОВЫЕ СТРУКТУРЫ ДЛЯ SOCIAL RECOVERY ==========

	/// Конфигурация системы восстановления
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct RecoveryConfig {
		/// Минимальное количество шаров для восстановления
		pub threshold: u8,
		/// Общее количество доверенных контактов
		pub total_trustees: u8,
		/// Период ожидания перед выполнением восстановления (блоки)
		pub delay_period: u32,
		/// Активна ли система восстановления
		pub active: bool,
		/// Депозит, заблокированный за настройку
		pub deposit: BalanceOf<Self>,
	}

	/// Шар секрета для доверенного контакта
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct TrusteeShare {
		/// Аккаунт доверенного контакта
		pub trustee_account: <T as frame_system::Config>::AccountId,
		/// Зашифрованный шар секрета
		pub share: BoundedVec<u8, ConstU32<1024>>,
		/// Подтвердил ли контакт участие
		pub confirmed: bool,
	}

	/// Запрос на восстановление
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct RecoveryRequest {
		/// Аккаунт инициирующий восстановление
		pub requesting_account: <T as frame_system::Config>::AccountId,
		/// Новый публичный ключ
		pub new_public_key: [u8; 32],
		/// Количество собранных шаров
		pub submitted_shares: u8,
		/// Когда можно выполнить восстановление (номер блока)
		pub execute_at: u32,
		/// Завершено ли восстановление
		pub completed: bool,
	}

	// ========== НОВЫЕ STORAGE ДЛЯ SOCIAL RECOVERY ==========

	#[pallet::storage]
	#[pallet::getter(fn get_recovery_config)]
	/// Конфигурации восстановления для каждого аккаунта
	pub type RecoveryConfigs<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		RecoveryConfig,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_trustee_share)]
	/// Шары секрета для доверенных контактов
	pub type TrusteeShares<T: Config> = StorageDoubleMap<
		_, 
		Blake2_128Concat, 
		T::AccountId,           // Владелец
		Blake2_128Concat, 
		T::AccountId,           // Доверенный контакт
		TrusteeShare,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_active_recovery)]
	/// Активные запросы на восстановление
	pub type ActiveRecoveries<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		RecoveryRequest,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_recovery_deposit)]
	/// Заблокированные депозиты для восстановления
	pub type RecoveryDeposits<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AccountId, 
		BalanceOf<T>, 
		OptionQuery
	>;

	// ========== СОБЫТИЯ ==========

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Существующие события
		AuraIdCreated { account: T::AccountId, did: [u8; 32] },
		
		// Новые события для Social Recovery
		RecoveryConfigured { 
			account: T::AccountId, 
			threshold: u8, 
			total_trustees: u8 
		},
		TrusteeAdded { 
			account: T::AccountId, 
			trustee: T::AccountId 
		},
		TrusteeRemoved { 
			account: T::AccountId, 
			trustee: T::AccountId 
		},
		RecoveryInitiated { 
			lost_account: T::AccountId, 
			requesting_account: T::AccountId 
		},
		RecoveryShareProvided { 
			lost_account: T::AccountId, 
			trustee: T::AccountId 
		},
		RecoveryExecuted { 
			lost_account: T::AccountId, 
			new_account: T::AccountId 
		},
		RecoveryCancelled { 
			account: T::AccountId 
		},
	}

	// ========== ОШИБКИ ==========

	#[pallet::error]
	pub enum Error<T> {
		// Существующие ошибки
		AuraIdAlreadyExists,
		AuraIdNotFound,
		
		// Новые ошибки для Social Recovery
		InvalidRecoveryThreshold,
		TooManyTrustees,
		TrusteeNotFound,
		RecoveryAlreadyConfigured,
		RecoveryNotConfigured,
		RecoveryAlreadyActive,
		RecoveryNotActive,
		InsufficientShares,
		DelayPeriodNotPassed,
		InsufficientDeposit,
		NotAuthorized,
	}

	// ========== CALL ФУНКЦИИ ==========

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// СУЩЕСТВУЮЩАЯ ФУНКЦИЯ - обновлена
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

			let bounded_recovery_config: BoundedVec<u8, ConstU32<1024>> = recovery_config
				.try_into()
				.map_err(|_| "Recovery config too large")?;

			let record = AuraIdRecord {
				did,
				public_key,
				recovery_config: bounded_recovery_config,
				created: 0u32, // Will implement proper timestamp later
			};

			AuraIdentities::<T>::insert(&who, record.clone());
			DidIndex::<T>::insert(did, &who);

			Self::deposit_event(Event::AuraIdCreated { account: who, did });

			Ok(())
		}

		// ОБНОВЛЕННАЯ ФУНКЦИЯ - теперь полноценная Social Recovery
		#[pallet::call_index(1)]
		#[pallet::weight(50_000)]
		pub fn setup_recovery(
			origin: OriginFor<T>,
			threshold: u8,
			trustees: Vec<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(!RecoveryConfigs::<T>::contains_key(&who), Error::<T>::RecoveryAlreadyConfigured);
			
			// Проверяем порог
			ensure!(
				threshold >= MIN_THRESHOLD && threshold <= MAX_THRESHOLD,
				Error::<T>::InvalidRecoveryThreshold
			);
			
			let total_trustees = trustees.len() as u8;
			ensure!(
				total_trustees >= threshold && total_trustees <= T::MaxTrustees::get() as u8,
				Error::<T>::TooManyTrustees
			);
			
			// Проверяем, что все доверенные контакты имеют Aura ID
			for trustee in &trustees {
				ensure!(AuraIdentities::<T>::contains_key(trustee), Error::<T>::AuraIdNotFound);
			}
			
			// Блокируем депозит
			let deposit = T::RecoveryDeposit::get();
			T::Currency::reserve(&who, deposit)?;
			
			// Создаем конфигурацию
			let config = RecoveryConfig {
				threshold,
				total_trustees,
				delay_period: DEFAULT_RECOVERY_DELAY,
				active: true,
				deposit,
			};
			
			RecoveryConfigs::<T>::insert(&who, config);
			RecoveryDeposits::<T>::insert(&who, deposit);
			
			// Сохраняем доверенные контакты
			for trustee in trustees {
				let share = TrusteeShare {
					trustee_account: trustee.clone(),
					share: BoundedVec::default(),
					confirmed: false,
				};
				TrusteeShares::<T>::insert(&who, &trustee, share);
				
				Self::deposit_event(Event::TrusteeAdded { 
					account: who.clone(), 
					trustee 
				});
			}
			
			Self::deposit_event(Event::RecoveryConfigured { 
				account: who, 
				threshold, 
				total_trustees 
			});
			
			Ok(())
		}

		// НОВАЯ ФУНКЦИЯ - добавление доверенного контакта
		#[pallet::call_index(2)]
		#[pallet::weight(30_000)]
		pub fn add_trustee(
			origin: OriginFor<T>,
			trustee: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let mut config = RecoveryConfigs::<T>::get(&who)
				.ok_or(Error::<T>::RecoveryNotConfigured)?;
			
			// Проверяем лимит доверенных контактов
			ensure!(
				config.total_trustees < T::MaxTrustees::get() as u8,
				Error::<T>::TooManyTrustees
			);
			
			ensure!(AuraIdentities::<T>::contains_key(&trustee), Error::<T>::AuraIdNotFound);
			
			// Проверяем, что контакт еще не добавлен
			ensure!(
				!TrusteeShares::<T>::contains_key(&who, &trustee),
				Error::<T>::TrusteeNotFound
			);
			
			// Добавляем доверенный контакт
			let share = TrusteeShare {
				trustee_account: trustee.clone(),
				share: BoundedVec::default(),
				confirmed: false,
			};
			
			TrusteeShares::<T>::insert(&who, &trustee, share);
			config.total_trustees += 1;
			RecoveryConfigs::<T>::insert(&who, config);
			
			Self::deposit_event(Event::TrusteeAdded { 
				account: who, 
				trustee 
			});
			
			Ok(())
		}

		// НОВАЯ ФУНКЦИЯ - удаление доверенного контакта
		#[pallet::call_index(3)]
		#[pallet::weight(30_000)]
		pub fn remove_trustee(
			origin: OriginFor<T>,
			trustee: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			let mut config = RecoveryConfigs::<T>::get(&who)
				.ok_or(Error::<T>::RecoveryNotConfigured)?;
			
			// Проверяем, что контакт существует
			ensure!(
				TrusteeShares::<T>::contains_key(&who, &trustee),
				Error::<T>::TrusteeNotFound
			);
			
			// Удаляем контакт
			TrusteeShares::<T>::remove(&who, &trustee);
			config.total_trustees -= 1;
			
			// Если количество контактов стало меньше порога, обновляем порог
			if config.threshold > config.total_trustees {
				config.threshold = config.total_trustees;
			}
			
			RecoveryConfigs::<T>::insert(&who, config);
			
			Self::deposit_event(Event::TrusteeRemoved { 
				account: who, 
				trustee 
			});
			
			Ok(())
		}

		// НОВАЯ ФУНКЦИЯ - инициация восстановления
		#[pallet::call_index(4)]
		#[pallet::weight(40_000)]
		pub fn initiate_recovery(
			origin: OriginFor<T>,
			lost_account: T::AccountId,
			new_public_key: [u8; 32],
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			// Проверяем, что потерянный аккаунт существует
			ensure!(AuraIdentities::<T>::contains_key(&lost_account), Error::<T>::AuraIdNotFound);
			
			// Проверяем, что система восстановления настроена
			let config = RecoveryConfigs::<T>::get(&lost_account)
				.ok_or(Error::<T>::RecoveryNotConfigured)?;
			
			ensure!(config.active, Error::<T>::RecoveryNotConfigured);
			
			// Проверяем, что восстановление еще не активно
			ensure!(
				!ActiveRecoveries::<T>::contains_key(&lost_account),
				Error::<T>::RecoveryAlreadyActive
			);
			
			// Создаем запрос на восстановление
			let recovery_request = RecoveryRequest {
				requesting_account: who.clone(),
				new_public_key,
				submitted_shares: 0,
				execute_at: frame_system::Pallet::<T>::block_number() + config.delay_period,
				completed: false,
			};
			
			ActiveRecoveries::<T>::insert(&lost_account, recovery_request);
			
			Self::deposit_event(Event::RecoveryInitiated { 
				lost_account, 
				requesting_account: who 
			});
			
			Ok(())
		}
	}

	// ========== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ==========

	impl<T: Config> Pallet<T> {
		pub fn generate_did(public_key: &[u8; 32]) -> [u8; 32] {
			sp_io::hashing::blake2_256(public_key)
		}
	}

	// ========== СУЩЕСТВУЮЩАЯ СТРУКТУРА ==========

	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
	pub struct AuraIdRecord {
		pub did: [u8; 32],
		pub public_key: [u8; 32],
		pub recovery_config: BoundedVec<u8, ConstU32<1024>>,
		pub created: u32,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{
		assert_noop, assert_ok, 
		traits::{ConstU32, Currency, ReservableCurrency},
		BoundedVec
	};
	use sp_core::H256;
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup},
		BuildStorage, DispatchError,
	};
	use frame_system::pallet_prelude::BlockNumberFor;

	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test {
			System: frame_system,
			AuraIdentity: pallet,
		}
	);

	// Конфигурация для тестов
	pub struct TestMaxTrustees;
	impl Get<u32> for TestMaxTrustees {
		fn get() -> u32 { 5 }
	}

	pub struct TestRecoveryDeposit;
	impl Get<u128> for TestRecoveryDeposit {
		fn get() -> u128 { 100 }
	}

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
		type BlockHashCount = frame_support::traits::ConstU64<250>;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<u128>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_balances::Config for Test {
		type Balance = u128;
		type RuntimeEvent = RuntimeEvent;
		type DustRemoval = ();
		type ExistentialDeposit = ConstU128<1>;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = ();
		type MaxReserves = ();
		type ReserveIdentifier = [u8; 8];
		type HoldIdentifier = ();
		type FreezeIdentifier = ();
		type MaxHolds = ();
		type MaxFreezes = ();
	}

	impl Config for Test {
		type RuntimeEvent = RuntimeEvent;
		type Currency = Balances;
		type MaxTrustees = TestMaxTrustees;
		type RecoveryDeposit = TestRecoveryDeposit;
	}

	// СУЩЕСТВУЮЩИЕ ТЕСТЫ
	#[test]
	fn test_did_generation() {
		let public_key = [1u8; 32];
		let did = Pallet::<Test>::generate_did(&public_key);

		assert_eq!(did.len(), 32);
		assert_ne!(did, public_key);
	}

	#[test]
	fn test_create_aura_id() {
		new_test_ext().execute_with(|| {
			let account_id = 1;
			let public_key = [2u8; 32];
			let recovery_config = vec![1, 2, 3];

			assert_ok!(AuraIdentity::create_aura_id(
				RuntimeOrigin::signed(account_id),
				public_key,
				recovery_config.clone()
			));

			let record = AuraIdentity::get_aura_id(account_id).unwrap();
			assert_eq!(record.public_key, public_key);
			let expected_bounded: BoundedVec<u8, ConstU32<1024>> = recovery_config.try_into().unwrap();
			assert_eq!(record.recovery_config, expected_bounded);
		});
	}

	#[test]
	fn test_duplicate_aura_id() {
		new_test_ext().execute_with(|| {
			let account_id = 1;
			let public_key = [3u8; 32];
			let recovery_config = vec![1, 2, 3];

			assert_ok!(AuraIdentity::create_aura_id(
				RuntimeOrigin::signed(account_id),
				public_key,
				recovery_config.clone()
			));

			assert_noop!(
				AuraIdentity::create_aura_id(
					RuntimeOrigin::signed(account_id),
					public_key,
					recovery_config
				),
				Error::<Test>::AuraIdAlreadyExists
			);
		});
	}

	// НОВЫЕ ТЕСТЫ ДЛЯ SOCIAL RECOVERY
	#[test]
	fn test_setup_recovery() {
		new_test_ext().execute_with(|| {
			// Создаем аккаунты
			let alice = 1;
			let bob = 2;
			let charlie = 3;
			
			// Создаем Aura ID для всех участников
			create_aura_id_for_account(alice);
			create_aura_id_for_account(bob);
			create_aura_id_for_account(charlie);
			
			// Настраиваем recovery систему для Alice
			assert_ok!(AuraIdentity::setup_recovery(
				RuntimeOrigin::signed(alice),
				2,
				vec![bob, charlie]
			));
			
			// Проверяем, что конфигурация создана
			assert!(AuraIdentity::get_recovery_config(alice).is_some());
			
			let config = AuraIdentity::get_recovery_config(alice).unwrap();
			assert_eq!(config.threshold, 2);
			assert_eq!(config.total_trustees, 2);
			assert!(config.active);
		});
	}

	#[test]
	fn test_add_remove_trustee() {
		new_test_ext().execute_with(|| {
			let alice = 1;
			let bob = 2;
			let charlie = 3;
			let dave = 4;
			
			create_aura_id_for_account(alice);
			create_aura_id_for_account(bob);
			create_aura_id_for_account(charlie);
			create_aura_id_for_account(dave);
			
			// Сначала настраиваем базовую систему
			assert_ok!(AuraIdentity::setup_recovery(
				RuntimeOrigin::signed(alice),
				2,
				vec![bob, charlie]
			));
			
			// Добавляем нового доверенного контакта
			assert_ok!(AuraIdentity::add_trustee(
				RuntimeOrigin::signed(alice),
				dave
			));
			
			// Проверяем, что контакт добавлен
			assert!(AuraIdentity::get_trustee_share(alice, dave).is_some());
			
			let config = AuraIdentity::get_recovery_config(alice).unwrap();
			assert_eq!(config.total_trustees, 3);
			
			// Удаляем доверенный контакт
			assert_ok!(AuraIdentity::remove_trustee(
				RuntimeOrigin::signed(alice),
				dave
			));
			
			// Проверяем, что контакт удален
			assert!(AuraIdentity::get_trustee_share(alice, dave).is_none());
		});
	}

	// Вспомогательная функция для создания Aura ID
	fn create_aura_id_for_account(account: u64) {
		let _ = AuraIdentity::create_aura_id(
			RuntimeOrigin::signed(account),
			[account as u8; 32],
			vec![],
		);
	}

	// Вспомогательная функция для тестов
	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		t.into()
	}
}
