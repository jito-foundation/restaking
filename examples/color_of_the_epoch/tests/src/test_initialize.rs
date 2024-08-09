use color_of_the_epoch::InitializeColorOfTheEpochParams;
use solana_program_test::*;

use crate::fixture;

#[tokio::test]
async fn test_initialize() {
    let fixture = fixture::TestFixture::new().await;
    fixture
        .initialize_color_of_the_epoch(InitializeColorOfTheEpochParams {
            color_coin_decimals: 9,
            slots_per_epoch: 100,
        })
        .await;

    println!("color_of_the_epoch: {:?}", fixture.color_of_the_epoch);

    // let accounts = InitializeColorOfTheEpoch {
    //     color_of_the_epoch,
    //     system_program,
    //     authority,
    // };

    // let ix = program.request().accounts(accounts).instructions();
}
