use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{
    load_signer, load_system_account, load_system_program, load_token_program,
};
use jito_vault_core::{loader::load_vault, vault::Vault};
use solana_program::{
    account_info::AccountInfo, borsh1::get_instance_packed_len, entrypoint::ProgramResult, msg,
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
    let [metadata_info, vault_info, vault_admin, system_program, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    msg!("System Account");
    load_system_account(metadata_info, true)?;
    msg!("Vault");
    load_vault(program_id, vault_info, false)?;
    msg!("Signer");
    load_signer(vault_admin, true)?;
    msg!("System program");
    load_system_program(system_program)?;
    msg!("Token program");
    load_token_program(token_program)?;

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice(&mut vault_data)?;

    let (vault_pubkey, _vault_bump, _vault_seeds) =
        Vault::find_program_address(program_id, &vault.base);
    msg!("Metadata info: {}", metadata_info.key);
    msg!("Vault admin info: {}", vault_admin.key);

    msg!(
        "Creating token metadata for token @ address {}",
        vault.lrt_mint
    );

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
    // let signer_seeds = &[vec![b"metadata"], mint_info.as_ref(), vec![]];
    seeds.push(vec![bump]);

    msg!("Creating an account for metadata",);
    jito_jsm_core::create_account(
        vault_admin,
        metadata_info,
        system_program,
        program_id,
        &Rent::get()?,
        space as u64,
        &seeds,
    )?;

    msg!("Instance size");
    let instance_size = get_instance_packed_len(&token_metadata)?;

    msg!("Buffer");
    // allocate a TLV entry for the space and write it in
    let mut buffer = metadata_info.try_borrow_mut_data()?;
    msg!("State");
    let mut state = TlvStateMut::unpack(&mut buffer)?;
    msg!("allocation");
    state.alloc::<TokenMetadata>(instance_size, false)?;
    msg!("pack_first");
    state.pack_first_variable_len_value(&token_metadata)?;
    msg!("done");
    // let token_mint_authority_signer_seeds: &[&[_]] = &[
    //     stake_pool_info.key.as_ref(),
    //     AUTHORITY_WITHDRAW,
    //     &[stake_withdraw_bump_seed],
    // ];

    // invoke(
    //     &ix,
    //     &[
    //         metadata.clone(),
    //         update_authority_info.clone(),
    //         mint_info.clone(),
    //         mint_authority_info.clone(),
    //     ],
    // )?;

    Ok(())
}
