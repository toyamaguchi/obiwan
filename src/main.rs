mod obiwan;

#[tokio::main]
async fn main() {
    obiwan::start().await;
}
