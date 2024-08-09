# Color of the Epoch

The Color of the Epoch NCN (Node Consensus Network) is a network that decides an official color each epoch in hex code (0xRRGGBB) and publishes it in a Solana color account.

The purpose of this program is to show how to create an NCN, and how it works with the Restaking program.

## NCN

The NCN will work as such:

At the beginning of each epoch, 3 operators ( specified in the NCN Config ) will upload a unique color code, and casting one vote per their own code. Any subsequent operator will vote for their favorite color. At the end of the voting period, the color with the most votes wins. A tie will go to the earliest uploaded color.

Rewards will be in `Color Coins`

Cost:
`submitting`: 0-100 Color Coins

Rewards payout:
`voting`:     10  x amount_delegated
`cranking`:    1  x amount_delegated
`winning`:     upload_bet x amount_delegated

Slashing:
`not voting`: 5% of amount_delegated

Structs:

```rust
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub votes: u8,
    pub cost: u32
}
```

```rust
pub struct ColorAccount {
    pub color_token_mint: u64,
    pub max_bet: u64,

    pub slots_per_epoch: u64,
    pub last_updated_epoch: u64,

    pub color_of_the_epoch: Color,
    pub actively_voting_colors: [3: Color]
}
```

```rust
ColorOperator {
    last_updated_slot: u64,

    upload_index: Option<u64>,
    upload_bet: u64,

    crank_count: u64,
}
```

Functions
`initialize_color_account`
`submit_and_vote`
`submit_vote`
`distribute_crank`
