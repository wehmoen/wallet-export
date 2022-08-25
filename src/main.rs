mod roninrest;

use roninrest::Adapter;

enum NFT {
    Axie,
    Land,
    Item
}


#[tokio::main]
async fn main() {

    let rr = Adapter::new();

    let data = rr.list_nft(NFT::Item, "ronin:3759468f9fd589665c8affbe52414ef77f863f72".to_string()).await;

    println!("{:?}", data);

}
