use std::env;

use buy_me_a_coffee::MemberStatus;
use dotenvy::dotenv;

fn test_pat() -> String {
    dotenv().ok();

    env::var("TEST_PAT").expect("a personal access token must be provided to run tests")
}

#[tokio::test]
#[should_panic = "Client(401)"]
async fn unauthorized_error() {
    let client = buy_me_a_coffee::Client::new("invalid access token");

    client.members(MemberStatus::All, 1).await.unwrap();
}

#[tokio::test]
#[should_panic = "Client(404)"]
async fn not_found_error() {
    let client = buy_me_a_coffee::Client::new(&test_pat());

    client.membership(0).await.unwrap();
}

#[tokio::test]
#[should_panic = "No subscriptions"]
async fn no_subscriptions_error() {
    let client = buy_me_a_coffee::Client::new(&test_pat());

    client.members(MemberStatus::All, 1).await.unwrap();
}

#[tokio::test]
#[should_panic = "No supporters"]
async fn no_supporters_error() {
    let client = buy_me_a_coffee::Client::new(&test_pat());

    client.supporters(1).await.unwrap();
}
