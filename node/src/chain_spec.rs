use aura_chain_runtime::{
    AccountId, AuraIdentityConfig, BalancesConfig, RuntimeGenesisConfig, Signature, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, AccountId) {
    (get_from_seed::<AuraId>(s), get_account_id_from_seed::<sr25519::Public>(s))
}

/// Development keys for testing
pub fn development_authority_keys() -> Vec<(AuraId, AccountId)> {
    vec![
        authority_keys_from_seed("Alice"),
        authority_keys_from_seed("Bob"),
    ]
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, AccountId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> RuntimeGenesisConfig {
    RuntimeGenesisConfig {
        system: SystemConfig {
            // Add wasm runtime to storage.
            code: wasm_binary.to_vec(),
            ..Default::default()
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1_000_000 * aura_chain_runtime::constants::currency::UNIT))
                .collect(),
        },
        aura: Default::default(),
        grandpa: Default::default(),
        sudo: SudoConfig {
            // Assign network admin rights to root key.
            key: Some(root_key),
        },
        transaction_payment: Default::default(),
        aura_identity: AuraIdentityConfig {
            // Initialize identity pallet
            identities: Default::default(),
            next_identity_id: Default::default(),
        },
    }
}

/// Development config (single authority Alice)
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial authorities
                vec![authority_keys_from_seed("Alice")],
                // Root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some("aura-dev"),
        // Fork ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

/// Local testnet config (multi authority Alice + Bob)
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial authorities
                development_authority_keys(),
                // Root key
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Endowed accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some("aura-local"),
        // Fork ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}
