use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_signer, load_system_account, load_system_program};
use jito_vault_core::{loader::load_vault, vault::Vault};
use solana_program::{
    account_info::AccountInfo, borsh1::get_instance_packed_len, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::state::TlvStateMut;

pub fn process_create_token_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let [metadata_info, vault_info, vault_admin, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_system_account(metadata_info, true)?;
    load_vault(program_id, vault_info, false)?;
    load_signer(vault_admin, true)?;
    load_system_program(system_program)?;

    let vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice(&vault_data)?;

    let (vault_pubkey, _vault_bump, _vault_seeds) =
        Vault::find_program_address(program_id, &vault.base);

    let token_metadata = TokenMetadata {
        update_authority: OptionalNonZeroPubkey(vault_pubkey),
        mint: vault.lrt_mint,
        name,
        symbol,
        uri,
        ..Default::default()
    };

    let space = token_metadata.tlv_size_of()?;
    let mut seeds = vec![
        b"metadata".as_ref().to_vec(),
        vault.lrt_mint.to_bytes().to_vec(),
    ];
    let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
    let (_, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
    seeds.push(vec![bump]);

    jito_jsm_core::create_account(
        vault_admin,
        metadata_info,
        system_program,
        program_id,
        &Rent::get()?,
        space as u64,
        &seeds,
    )?;

    let instance_size = get_instance_packed_len(&token_metadata)?;

    // allocate a TLV entry for the space and write it in
    let mut buffer = metadata_info.try_borrow_mut_data()?;
    let mut state = TlvStateMut::unpack(&mut buffer)?;
    state.alloc::<TokenMetadata>(instance_size, false)?;
    state.pack_first_variable_len_value(&token_metadata)?;

    Ok(())
}
