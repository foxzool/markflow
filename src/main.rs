use markflow::cli;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::run().await {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
