use crate::cache::{Action, BatchActionArgs, RunActionArgs};
use crate::decode::get_metadata_pda;
use crate::errors::ActionError;
use crate::parse::parse_keypair;
use crate::{constants::*, parse::parse_solana_config};
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use mpl_token_metadata::{
    instruction::update_metadata_accounts_v2, ID as TOKEN_METADATA_PROGRAM_ID,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::str::FromStr;
use std::sync::Arc;

pub struct SetPrimarySaleHappenedAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub retries: u8,
}

struct SetPrimarySaleHappenedArgs {
    client: Arc<RpcClient>,
    keypair: Arc<Keypair>,
    mint_account: String,
}

pub fn set_primary_sale_happened_one(
    client: RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts_v2(
        program_id,
        metadata_account,
        update_authority,
        None,
        None,
        Some(true),
        None,
    );
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    info!("Tx sig: {:?}", sig);
    println!("Tx sig: {:?}", sig);

    Ok(())
}

async fn set_primary_sale_happened(args: SetPrimarySaleHappenedArgs) -> Result<(), ActionError> {
    let mint_pubkey = Pubkey::from_str(&args.mint_account).expect("Invalid mint pubkey");
    let update_authority = args.keypair.pubkey();
    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts_v2(
        TOKEN_METADATA_PROGRAM_ID,
        metadata_account,
        update_authority,
        None,
        None,
        Some(true),
        None,
    );
    let recent_blockhash = args
        .client
        .get_latest_blockhash()
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&*args.keypair],
        recent_blockhash,
    );

    let sig = args
        .client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    info!("Tx sig: {:?}", sig);
    Ok(())
}

pub struct SetPrimarySaleHappenedAll {}

#[async_trait]
impl Action for SetPrimarySaleHappenedAll {
    fn name() -> &'static str {
        "set-primary-sale-happened-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        set_primary_sale_happened(SetPrimarySaleHappenedArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
        })
        .await
    }
}

pub async fn set_primary_sale_happened_all(args: SetPrimarySaleHappenedAllArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        mint_list: args.mint_list,
        cache_file: args.cache_file,
        new_value: "".to_string(),
        retries: args.retries,
    };
    SetPrimarySaleHappenedAll::run(args).await?;

    Ok(())
}
