use solana_remote_wallet::{
    ledger::get_ledger_from_info,
    remote_keypair::RemoteKeypair,
    remote_wallet::{initialize_wallet_manager, RemoteWalletType},
};
use solana_sdk::{
    derivation_path::DerivationPath,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer, SignerError},
};

pub struct CliSigner {
    pub keypair: Option<Keypair>,
    pub remote_keypair: Option<RemoteKeypair>,
}

impl CliSigner {
    pub fn new(keypair: Option<Keypair>, remote_keypair: Option<RemoteKeypair>) -> Self {
        if keypair.is_none() && remote_keypair.is_none() {
            panic!("No keypair or wallet manager provided");
        }

        Self {
            keypair,
            remote_keypair,
        }
    }

    pub fn new_keypair(keypair: Keypair) -> Self {
        Self::new(Some(keypair), None)
    }

    pub fn new_keypair_from_path(keypair_path: &str) -> Self {
        Self::new(
            Some(read_keypair_file(keypair_path).expect("No keypair found")),
            None,
        )
    }

    // Will only work with Ledger devices as
    pub fn new_ledger() -> Self {
        println!("\nConnecting to Ledger Device");
        println!("- This will only work with Ledger devices.");
        println!("- It will use the first account on the first connected Ledger.");
        println!("- The Ledger must be unlocked and the Solana app open.\n");

        println!("Searching for wallets...");
        let wallet_manager =
            initialize_wallet_manager().expect("Could not initialize wallet manager");
        let device_count = wallet_manager
            .update_devices()
            .expect("Could not fetch devices");
        println!("Wallet found with {} device(s) connected", device_count);

        let devices = wallet_manager.list_devices();
        let device = devices.first().expect("No devices found");
        let ledger = get_ledger_from_info(device.clone(), "Signer", &wallet_manager)
            .expect("This CLI only supports Ledger devices");
        let account_index = 0;
        let derivation_path = DerivationPath::new_bip44(Some(account_index), Some(0));
        let path = format!("{}{}", ledger.pretty_path, derivation_path.get_query());
        let confirm_key = true;
        let remote_keypair = RemoteKeypair::new(
            RemoteWalletType::Ledger(ledger),
            derivation_path,
            confirm_key,
            path.clone(),
        )
        .expect("Could not create remote keypair");
        println!(
            "\nConnected to first Ledger device\n- {}\n",
            remote_keypair.pubkey
        );

        Self::new(None, Some(remote_keypair))
    }
}

impl Signer for CliSigner {
    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        if let Some(keypair) = &self.keypair {
            Ok(keypair.pubkey())
        } else if let Some(remote_keypair) = &self.remote_keypair {
            Ok(remote_keypair.pubkey)
        } else {
            Err(SignerError::NoDeviceFound)
        }
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        if let Some(keypair) = &self.keypair {
            keypair.try_sign_message(message)
        } else if let Some(remote_keypair) = &self.remote_keypair {
            remote_keypair.try_sign_message(message)
        } else {
            Err(SignerError::NoDeviceFound)
        }
    }

    fn is_interactive(&self) -> bool {
        // Remote wallets are typically interactive, local keypairs are not
        self.remote_keypair.is_some()
    }
}
