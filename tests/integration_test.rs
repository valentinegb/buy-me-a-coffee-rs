use std::env;

use buy_me_a_coffee::MemberStatus;
use dotenvy::dotenv;

fn test_pat() -> String {
    dotenv().ok();

    env::var("TEST_PAT").expect("a personal access token must be provided to run tests")
}

#[tokio::test]
#[should_panic = "401"]
#[ignore = "unexpected behaviour from server"]
async fn client_error() {
    let client = buy_me_a_coffee::Client::new("invalid access token");

    client.members(MemberStatus::All).await.unwrap();
}
