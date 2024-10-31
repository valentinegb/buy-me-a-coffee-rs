# `buy_me_a_coffee`

A Rust crate for interacting with the [Buy Me a Coffee] API.

```rs
use buy_me_a_coffee::MemberStatus;

let client = buy_me_a_coffee::Client::new("personal access token here");

for membership in client.members(MemberStatus::All, 1).await.unwrap().data {
    //                                              ^ first page
    println!("{}", membership.payer_name);
}
```

Truthfully, **this crate is not really ready to be used**. It's not possible to
test most things when I don't have any supporters on [Buy Me a Coffee] since the
API returns a "No \[subscriptions|supporters|extra purchases]" error. If someone
would [buy me a coffee][Buy Me a Coffee personal page], that would really help
out with this project, wink wink...

[Buy Me a Coffee]: https://buymeacoffee.com
[Buy Me a Coffee personal page]: https://buymeacoffee.com/im_valentinegb