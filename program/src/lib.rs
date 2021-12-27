use borsh::{BorshDeserialize, BorshSerialize};
use util::{PostlyAccount, PostlyAccountIndex};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let index_account = next_account_info(accounts_iter)?;
    let post_account = next_account_info(accounts_iter)?;

    if index_account.owner != program_id || post_account.owner != program_id {
        msg!("account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut index = PostlyAccountIndex::try_from_slice(*index_account.try_borrow_data()?)?;
    index.post_n += 1;
    index.serialize(&mut &mut index_account.data.borrow_mut()[..])?;

    let post = PostlyAccount::try_from_slice(instruction_data)?;
    post.serialize(&mut &mut post_account.data.borrow_mut()[..])?;

    msg!("Index: {}! My Post: {}!", index.post_n, post.post);
    Ok(())
}
