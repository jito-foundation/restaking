# Jito CLI Demo

## Create a Vault

`56PEeW2jxT3y9zRkASmNkKhK9FrA9msjGUg3JAKTj3Us`

Create Vault

```bash
cargo run -- vault vault initialize So11111111111111111111111111111111111111112 100 150 200
```

Create Update State Tracker

```bash
cargo run -- vault vault initialize-update-state-tracker 56PEeW2jxT3y9zRkASmNkKhK9FrA9msjGUg3JAKTj3Us
```

Close Vault Update State Tracker

```bash
cargo run -- vault vault close-update-state-tracker 56PEeW2jxT3y9zRkASmNkKhK9FrA9msjGUg3JAKTj3Us
```

```
CREATE VAULT AND MINT
getInitializeVaultInstruction
getInitializeVaultUpdateStateTrackerInstruction (simulate cranker)
getCloseVaultUpdateStateTrackerInstruction (simulate cranker)
getMintToInstruction

CREATING AND DELEGATING TO AN OPERATOR

create an operator
getInitializeOperatorVaultTicketInstruction

link a vault and operator
getInitializeOperatorVaultTicketInstruction
getWarmupOperatorVaultTicketInstruction

delegate to an operator
getInitializeVaultOperatorDelegationInstruction (may require hack to remove system program id from sdk
getAddDelegationInstruction)



WITHDRAW FROM VAULT
getEnqueueWithdrawalInstruction
getCrankVaultUpdateStateTrackerInstruction (to get rewards - not necessary)
getBurnWithdrawTicketInstruction (you need to wait at least an eopoch after getEnqueueWithdrawalInstruction was called for it to work)
```
