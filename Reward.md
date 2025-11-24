# Reward a Vault

Clone the repo:
```bash
git clone https://github.com/jito-foundation/restaking.git
cd restaking
git checkout ck/rewards
```

Navigate to CLI:
```bash
cd cli
```

Run the command to reward a vault:
```bash
cargo run -- --rpc-url <RPC_URL> --keypair <KEYPAIR_PATH> vault vault reward-vault <VAULT> <AMOUNT>
```

This function will send 96% of the rewards to the vault and 4% to the DAO. And then call `vault-update-balance`.

**NOTE**: The keypair that you use should have the tokens to reward the vault. And the `<AMOUNT>` is denoted in its lowest denomination. For example, if you want to reward 1 JTO, you should use `1_000_000_000` ( 9 decimals ) as the amount. 4% of the reward will be sent to the DAO.

**IMPORTANT**: It is highly recommended to try a test transaction of `10_000` tokens ( which if it has 9 decimals like solana you'd be sending 0.000_01 tokens ). `9600`, should be sent to the vault, `400` to the DAO.

## Example Transaction

Here is a test transaction I've run with my local developer wallet `HMRjDC8YYLxRsuxMrD3DhGit3etuoBZHMyv8yvhDkpzk` which has some `JTO` ( `jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL` ) and rewarding Kyros JTO vault ( `ABsoYTwRPBJEf55G7N8hVw7tQnDKBA6GkZCKBVrjTTcf` )

I am sending 1 JTO ( 1_000_000_000 ) as a reward to the vault.

```bash
cargo run vault vault reward-vault ABsoYTwRPBJEf55G7N8hVw7tQnDKBA6GkZCKBVrjTTcf 1000000000
```

[Result](https://explorer.solana.com/tx/4GJ4r3fFwmt47hXh5NRxq1mA3PimofeLVwApvJbFXhBmEC358xa6aBkCVMPQYSyCMeLGo6Dg9KLXVGruVsvxiQQR)
