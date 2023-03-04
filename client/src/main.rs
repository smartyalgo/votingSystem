use sp_keyring::AccountKeyring;
use subxt::{
	tx::PairSigner,
	OnlineClient,
	SubstrateConfig,
};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod substrate {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// tracing_subscriber::fmt::init();
	get_block_hash(1).await?;
	Ok(())
}

async fn get_block_hash(block_number: u32) -> Result<(), Box<dyn std::error::Error>> {
	// Create a client to use:
	let api = OnlineClient::<SubstrateConfig>::new().await?;
	let block_hash = api.rpc().block_hash(Some(block_number.into())).await?;

	if let Some(hash) = block_hash {
		println!("Block hash for block number {block_number}: {hash}");
	} else {
		println!("Block number {block_number} not found.");
	}

	Ok(())
}