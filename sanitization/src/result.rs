use solana_program::program_error::ProgramError;

pub type SanitizationResult<T> = Result<T, SanitizationError>;

#[derive(Debug)]
pub enum SanitizationError {
    AssociatedTokenAccountInvalidAddress,
    AssociatedTokenAccountInvalidOwner,
    AssociatedTokenAccountInvalidAccountData,
    AssociatedTokenAccountFailedReload,

    EmptyAccountNotWritable,
    EmptyAccountNotEmpty,

    SignerExpectedWritable,
    SignerNotSigner,

    SystemProgramInvalidAddress,

    TokenAccountInvalidAccountData,
    TokenAccountInvalidProgramOwner,
    TokenAccountInvalidMint,
    TokenAccountInvalidOwner,

    TokenMintExpectedWritable,
    TokenMintInvalidAccountData,
    TokenMintInvalidProgramOwner,

    TokenProgramInvalidAddress,
}

impl From<SanitizationError> for ProgramError {
    fn from(value: SanitizationError) -> Self {
        match value {
            SanitizationError::AssociatedTokenAccountInvalidAddress => Self::Custom(0),
            SanitizationError::AssociatedTokenAccountInvalidOwner => Self::Custom(1),
            SanitizationError::AssociatedTokenAccountInvalidAccountData => Self::Custom(2),
            SanitizationError::AssociatedTokenAccountFailedReload => Self::Custom(3),

            SanitizationError::EmptyAccountNotWritable => Self::Custom(100),
            SanitizationError::EmptyAccountNotEmpty => Self::Custom(101),

            SanitizationError::SignerExpectedWritable => Self::Custom(200),
            SanitizationError::SignerNotSigner => Self::Custom(201),

            SanitizationError::SystemProgramInvalidAddress => Self::Custom(300),

            SanitizationError::TokenAccountInvalidAccountData => Self::Custom(400),
            SanitizationError::TokenAccountInvalidProgramOwner => Self::Custom(401),
            SanitizationError::TokenAccountInvalidMint => Self::Custom(402),
            SanitizationError::TokenAccountInvalidOwner => Self::Custom(403),

            SanitizationError::TokenMintExpectedWritable => Self::Custom(500),
            SanitizationError::TokenMintInvalidAccountData => Self::Custom(501),
            SanitizationError::TokenMintInvalidProgramOwner => Self::Custom(502),

            SanitizationError::TokenProgramInvalidAddress => Self::Custom(600),
        }
    }
}
