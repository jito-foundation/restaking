use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{
    load_signer, load_system_account, load_system_program, load_token_mint,
};
use jito_vault_core::{loader::load_mpl_metadata_program, vault::Vault};
use jito_vault_sdk::inline_mpl_token_metadata::{
    instruction::create_metadata_accounts_v3, pda::find_metadata_account,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

pub fn process_create_token_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let [vault_info, admin, vrt_mint, payer, metadata, mpl_token_metadata_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Vault::load(program_id, vault_info, false)?;
    let vault_data = vault_info.data.borrow_mut();
    let vault: &Vault = Vault::try_from_slice_unchecked(&vault_data)?;
    load_signer(admin, false)?;
    load_token_mint(vrt_mint)?;
    load_signer(payer, true)?;
    load_system_account(metadata, true)?;
    load_mpl_metadata_program(mpl_token_metadata_program)?;
    load_system_program(system_program)?;

    vault.check_admin(admin.key)?;
    vault.check_vrt_mint(vrt_mint.key)?;

    let (metadata_account_pubkey, _) = find_metadata_account(vrt_mint.key);
    if metadata_account_pubkey != *metadata.key {
        msg!("Metadata account PDA does not match");
        return Err(ProgramError::InvalidAccountData);
    }

    let new_metadata_instruction = create_metadata_accounts_v3(
        *mpl_token_metadata_program.key,
        *metadata.key,
        *vrt_mint.key,
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
            vrt_mint.clone(),
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
