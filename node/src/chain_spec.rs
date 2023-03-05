use node_template_runtime::{
	pallet_voting_system::GenesisConfig as VotingSystemConfig, AccountId, AuraConfig,
	GenesisConfig, GrandpaConfig, Signature, SudoConfig, SystemConfig, WASM_BINARY,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	let candidate1_account_id = get_account_id_from_seed::<sr25519::Public>("Candidate1");
	let candidate2_account_id = get_account_id_from_seed::<sr25519::Public>("Candidate2");
	let candidate3_account_id = get_account_id_from_seed::<sr25519::Public>("Candidate3");

	let candidate1_public_key = vec![1, 2, 3];
	let candidate2_public_key = vec![1, 2, 3];
	let candidate3_public_key = vec![1, 2, 3];

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Central authority
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Candidates
				vec![
					(candidate1_account_id.clone(), candidate1_public_key.clone()),
					(candidate2_account_id.clone(), candidate2_public_key.clone()),
					(candidate3_account_id.clone(), candidate3_public_key.clone()),
				],
				// Ballot public key
				get_account_id_from_seed::<sr25519::Public>("ballot").to_string().into_bytes(),
				// sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	let candidate1_account_id = get_account_id_from_seed::<sr25519::Public>("Candidate1");
	let candidate2_account_id = get_account_id_from_seed::<sr25519::Public>("Candidate2");
	let candidate3_account_id = get_account_id_from_seed::<sr25519::Public>("Candidate3");

	let candidate1_public_key = vec![1, 2, 3];
	let candidate2_public_key = vec![1, 2, 3];
	let candidate3_public_key = vec![1, 2, 3];

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Central authority
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Candidates
				vec![
					(candidate1_account_id.clone(), candidate1_public_key.clone()),
					(candidate2_account_id.clone(), candidate2_public_key.clone()),
					(candidate3_account_id.clone(), candidate3_public_key.clone()),
				],
				// Ballot public key
				get_account_id_from_seed::<sr25519::Public>("ballot").to_string().into_bytes(),
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	central_authority: AccountId,
	candidates: Vec<(AccountId, Vec<u8>)>,
	ballot_public_key: Vec<u8>,
	root_key: AccountId,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key.clone()),
		},
		voting_system: VotingSystemConfig {
			central_authority: Some(central_authority),
			candidates,
			ballot_public_key,
		},
	}
}
