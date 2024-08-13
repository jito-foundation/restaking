use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{
    load_signer, load_system_account, load_system_program, load_token_mint,
};
use jito_vault_core::{
    loader::{load_mpl_metadata_program, load_vault},
    vault::Vault,
};
use jito_vault_sdk::inline_mpl_token_metadata::instruction::create_metadata_accounts_v3;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

pub fn process_create_token_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let [vault_info, admin, lrt_mint, payer, metadata, mpl_token_metadata_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_vault(program_id, vault_info, false)?;
    load_signer(admin, true)?;
    load_token_mint(lrt_mint)?;
    load_signer(payer, true)?;
    load_system_account(metadata, true)?;
    load_mpl_metadata_program(mpl_token_metadata_program)?;
    load_system_program(system_program)?;

    let vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice(&vault_data)?;

    vault.check_admin(admin)?;

    let new_metadata_instruction = create_metadata_accounts_v3(
        *mpl_token_metadata_program.key,
        *metadata.key,
        *lrt_mint.key,
        *vault_info.key,
        *payer.key,
        *vault_info.key,
        name,
        symbol,
        uri,
    );

    let (_vault_pubkey, vault_bump, mut vault_seeds) =
        Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);

    drop(vault_data);

    invoke_signed(
        &new_metadata_instruction,
        &[
            metadata.clone(),
            lrt_mint.clone(),
            vault_info.clone(),
            payer.clone(),
            vault_info.clone(),
            system_program.clone(),
        ],
        &[vault_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}
