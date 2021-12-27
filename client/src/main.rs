use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

use solana_client::rpc_client::RpcClient;
use std::env;
use util::{PostlyAccount, PostlyAccountIndex, PostlyError};

static INDEX_ACCOUNT_SEED: &str = "postly_index";
static POST_ACCOUNT_SEED: &str = "postly_account";

fn view(program_id: &Pubkey, keypair: &Keypair, rpc: &RpcClient) -> Result<(), PostlyError> {
    let index_account_adr =
        Pubkey::create_with_seed(&keypair.pubkey(), INDEX_ACCOUNT_SEED, &program_id)
            .map_err(|_| PostlyError::AccountError)?;

    let index_account = rpc
        .get_account(&index_account_adr)
        .map_err(|_| PostlyError::AccountError)?;

    let index = PostlyAccountIndex::try_from_slice(&index_account.data)
        .map_err(|_| PostlyError::AccountError)?;

    for id in 0..index.post_n {
        let post_acnt_seed = format!("{}_{}", POST_ACCOUNT_SEED, id);
        let post_acnt_addr =
            Pubkey::create_with_seed(&keypair.pubkey(), &post_acnt_seed, &program_id)
                .map_err(|_| PostlyError::AccountError)?;

        let post_account = rpc
            .get_account(&post_acnt_addr)
            .map_err(|_| PostlyError::PostError)?;
        let post = PostlyAccount::try_from_slice(&post_account.data)
            .map_err(|_| PostlyError::PostError)?;
        println!("My {} Post => {}", id, post.post);
    }
    Ok(())
}

fn post(
    content: String,
    program_id: &Pubkey,
    keypair: &Keypair,
    rpc: &RpcClient,
) -> Result<(), PostlyError> {
    let account_seed = "postly_index".to_string();
    let new_account = Pubkey::create_with_seed(&keypair.pubkey(), &account_seed, &program_id)
        .map_err(|_| PostlyError::AccountError)?;
    println!("Index Account: {}", new_account);

    let index_account = match rpc.get_account(&new_account) {
        Ok(acc_info) => PostlyAccountIndex::try_from_slice(&acc_info.data)
            .map_err(|_| PostlyError::AccountError)?,
        Err(_e) => {
            println!("Account doesn't exist, creating...");

            let index_acnt = PostlyAccountIndex { post_n: 0 };
            let mut i_acnt_buffer: Vec<u8> = Vec::new();
            index_acnt.serialize(&mut i_acnt_buffer).unwrap();

            let rent = rpc
                .get_minimum_balance_for_rent_exemption(i_acnt_buffer.len())
                .unwrap();
            let instruction = system_instruction::create_account_with_seed(
                &keypair.pubkey(),
                &new_account,
                &keypair.pubkey(),
                &account_seed,
                rent,
                i_acnt_buffer.len() as u64,
                &program_id,
            );
            let signers = [keypair];
            let instructions = vec![instruction];
            let (recent_hash, _) = rpc.get_recent_blockhash().unwrap();
            let txn = Transaction::new_signed_with_payer(
                &instructions,
                Some(&keypair.pubkey()),
                &signers,
                recent_hash,
            );
            let sig = rpc
                .send_and_confirm_transaction(&txn)
                .map_err(|_| PostlyError::AccountError)?;
            println!("created account {}", sig);
            index_acnt
        }
    };
    println!("my account: {:?}", index_account);

    let post_acnt = PostlyAccount { post: content };
    let mut p_acnt_bufffer: Vec<u8> = Vec::new();
    post_acnt
        .serialize(&mut p_acnt_bufffer)
        .map_err(|_| PostlyError::AccountError)?;

    let post_acnt_seed = format!("{}_{}", POST_ACCOUNT_SEED, index_account.post_n);
    let post_acnt_addr =
        Pubkey::create_with_seed(&keypair.pubkey(), &post_acnt_seed, &program_id).unwrap();
    let rent = rpc
        .get_minimum_balance_for_rent_exemption(p_acnt_bufffer.len())
        .unwrap();

    let acount_instruction = system_instruction::create_account_with_seed(
        &keypair.pubkey(),
        &post_acnt_addr,
        &keypair.pubkey(),
        &post_acnt_seed,
        rent,
        p_acnt_bufffer.len() as u64,
        &program_id,
    );
    let program_instruction = Instruction::new_with_borsh(
        *program_id,
        &post_acnt,
        vec![
            AccountMeta::new(new_account, false),
            AccountMeta::new(post_acnt_addr, false),
        ],
    );

    let instructions = vec![acount_instruction, program_instruction];
    let (recent_hash, _) = rpc.get_recent_blockhash().unwrap();
    let signers = [keypair];
    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &signers,
        recent_hash,
    );
    let sig = rpc
        .send_and_confirm_transaction(&txn)
        .map_err(|_| PostlyError::PostError)?;
    println!("Posted {:?}", sig);
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Nohting to do: {:?}", args);
        return;
    }
    let url = "http://127.0.0.1:8899".to_string();
    let program_id_str = "B1d7dDeTzrLxtHdaq1DsvbCqsoCVmpcdv4VfkZppC73Y".to_string();
    println!("Using sollan cluster {}", url);
    let rpc = RpcClient::new_with_commitment(url, CommitmentConfig::confirmed());
    let secret_key: [u8; 1] = [0];

    let keypair = Keypair::from_bytes(&secret_key).unwrap();
    let program_id = program_id_str.parse::<Pubkey>().unwrap();

    match args[1].as_ref() {
        "air_drop" => {
            let _ = rpc.request_airdrop(&keypair.pubkey(), 1000000000).unwrap();
        }
        "post" => {
            post(args[2].clone(), &program_id, &keypair, &rpc).unwrap();
        }
        "view" => {
            view(&program_id, &keypair, &rpc).unwrap();
        }
        _ => {
            println!("unknown command {}", args[1]);
        }
    }
}
