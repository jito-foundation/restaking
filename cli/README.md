# Jito CLI Demo

## Create a Vault

MINT = `So11111111111111111111111111111111111111112`
VAULT = `56PEeW2jxT3y9zRkASmNkKhK9FrA9msjGUg3JAKTj3Us`
OPERATOR = `7X6oxWuM2ENNc6BJerabBVF3F7MZ79gwT61rqP6LXxMS`

Create Vault

```bash
cargo run -- vault vault initialize MINT DEPOSIT_FEE_BPS WITHDRAWAL_FEE_BPS REWARD_FEE_BPS
```

Create Update State Tracker

```bash
cargo run -- vault vault initialize-update-state-tracker VAULT
```

Close Vault Update State Tracker

```bash
cargo run -- vault vault close-update-state-tracker VAULT
```

Mint VRT

```bash
cargo run -- vault vault mint-vrt VAULT AMOUNT_IN MIN_AMOUNT_OUT
```

Create an Operator

```bash
cargo run -- restaking operator initialize
```

Create Operator Vault Ticket

```bash
cargo run -- restaking operator initialize-operator-vault-ticket OPERATOR VAULT
```

Warmup Operator Vault Ticket

```bash
cargo run -- restaking operator warmup-operator-vault-ticket OPERATOR VAULT
```

Initialize Vault Operator Delegation

```bash
cargo run -- vault vault initialize-operator-delegation VAULT OPERATOR
```

Delegate to Operator

```bash
cargo run -- vault vault delegate-to-operator VAULT OPERATOR AMOUNT
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
