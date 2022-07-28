#[tokio::main]
pub async fn main() {
    endless_game::start().await.expect("Error occurred");
}
