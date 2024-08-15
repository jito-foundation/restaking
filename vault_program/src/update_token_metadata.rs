use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{
    loader::{load_mpl_metadata, load_mpl_metadata_program, load_vault},
    vault::Vault,
};
use jito_vault_sdk::inline_mpl_token_metadata::{
    instruction::update_metadata_accounts_v2, state::DataV2,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

pub fn process_update_token_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let [vault_info, admin, metadata, mpl_token_metadata_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_vault(program_id, vault_info, false)?;
    load_signer(admin, false)?;
    load_mpl_metadata_program(mpl_token_metadata_program)?;

    let vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice(&vault_data)?;

    load_mpl_metadata(metadata, &vault.vrt_mint)?;

    vault.check_admin(admin)?;

    let update_metadata_accounts_instruction = update_metadata_accounts_v2(
        *mpl_token_metadata_program.key,
        *metadata.key,
        *vault_info.key,
        None,
        Some(DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        }),
        None,
        Some(true),
    );

    let (_vault_pubkey, vault_bump, mut vault_seeds) =
        Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);

    drop(vault_data);

    invoke_signed(
        &update_metadata_accounts_instruction,
        &[metadata.clone(), vault_info.clone()],
        &[vault_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}
