use jito_restaking_sanitization::{
    empty_account::EmptyAccount, metadata_program::SanitizedMetadataProgram,
    signer::SanitizedSignerAccount, system_program::SanitizedSystemProgram,
    sysvar::SanitizedSysvar, token_mint::SanitizedTokenMint,
};
use jito_vault_core::vault::{SanitizedVault, Vault};
use mpl_token_metadata::{instructions::CreateV1CpiBuilder, types::TokenStandard};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_create_token_metadata(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let SanitizedAccounts {
        vault,
        lrt_mint,
        metadata,
        admin,
        system_program,
        metadata_program,
        sysvar,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    let mut builder = CreateV1CpiBuilder::new(metadata_program.account());
    let cpi_create = builder
        .metadata(metadata.account())
        .mint(lrt_mint.account(), false)
        .authority(vault.account())
        .payer(admin.account())
        .update_authority(vault.account(), false)
        .is_mutable(true)
        .primary_sale_happened(false)
        .seller_fee_basis_points(0)
        .name(name)
        .uri(uri)
        .symbol(symbol)
        .token_standard(TokenStandard::Fungible)
        .system_program(system_program.account())
        .sysvar_instructions(sysvar.account());

    let (_, bump, mut seeds) = Vault::find_program_address(program_id, &vault.vault().base());
    seeds.push(vec![bump]);
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    cpi_create.invoke_signed(&[&seed_slices])?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    lrt_mint: SanitizedTokenMint<'a, 'info>,
    metadata: EmptyAccount<'a, 'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
    system_program: SanitizedSystemProgram<'a, 'info>,
    metadata_program: SanitizedMetadataProgram<'a, 'info>,
    sysvar: SanitizedSysvar<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let mut accounts_iter = accounts.iter();

        let vault =
            SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, false)?;
        let lrt_mint = SanitizedTokenMint::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let metadata = EmptyAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let admin = SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
        let system_program =
            SanitizedSystemProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        let metadata_program =
            SanitizedMetadataProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
        let sysvar = SanitizedSysvar::sanitize(next_account_info(&mut accounts_iter)?)?;

        Ok(SanitizedAccounts {
            vault,
            lrt_mint,
            metadata,
            admin,
            system_program,
            metadata_program,
            sysvar,
        })
    }
}
