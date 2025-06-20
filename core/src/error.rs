use solana_decode_error::DecodeError;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("Bad epoch length")]
    BadEpochLength,
}

impl<T> DecodeError<T> for CoreError {
    fn type_of() -> &'static str {
        "jito::core"
    }
}

impl From<CoreError> for ProgramError {
    fn from(e: CoreError) -> Self {
        Self::Custom(e as u32)
    }
}

impl From<CoreError> for u64 {
    fn from(e: CoreError) -> Self {
        e as Self
    }
}

impl From<CoreError> for u32 {
    fn from(e: CoreError) -> Self {
        e as Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = CoreError::BadEpochLength;
        assert_eq!(error.to_string(), "Bad epoch length");
    }

    #[test]
    fn test_decode_error_type() {
        assert_eq!(
            <CoreError as DecodeError<CoreError>>::type_of(),
            "jito::core"
        );
    }

    #[test]
    fn test_conversion_to_program_error() {
        let error = CoreError::BadEpochLength;
        let program_error: ProgramError = error.into();

        // Verify the conversion creates a Custom program error with the correct error code
        match program_error {
            ProgramError::Custom(code) => {
                assert_eq!(code, CoreError::BadEpochLength as u32);
            }
            _ => panic!("Expected ProgramError::Custom"),
        }
    }

    #[test]
    fn test_conversion_to_u64() {
        let error = CoreError::BadEpochLength;
        let code: u64 = error.into();
        assert_eq!(code, CoreError::BadEpochLength as u64);
    }

    #[test]
    fn test_conversion_to_u32() {
        let error = CoreError::BadEpochLength;
        let code: u32 = error.into();
        assert_eq!(code, CoreError::BadEpochLength as u32);
    }

    #[test]
    fn test_error_equality() {
        let error1 = CoreError::BadEpochLength;
        let error2 = CoreError::BadEpochLength;
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_debug_implementation() {
        let error = CoreError::BadEpochLength;
        assert_eq!(format!("{:?}", error), "BadEpochLength");
    }
}
