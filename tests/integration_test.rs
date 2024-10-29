use buy_me_a_coffee::MemberStatus;

#[tokio::test]
#[should_panic = "401"]
#[ignore = "unexpected behaviour from server"]
async fn error() {
    let client = buy_me_a_coffee::Client::new("invalid access token");

    client.members(MemberStatus::All).await.unwrap();
}
