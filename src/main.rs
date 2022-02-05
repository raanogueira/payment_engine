use std::env;
use tokio::task;
mod exchange;

#[tokio::main]
async fn main() {
    let mut exchange = exchange::Exchange::new();
    if let Some(file) = env::args().nth(1) {
        let exchange = task::spawn_blocking(move || {
            if let Err(e) = exchange::process_transactions_from_csv(&file, &mut exchange) {
                eprintln!("Failed to read CSV with exception: {}", e)
            }
            exchange
        }).await.unwrap(); 
        
        exchange.to_csv()
    } else {
        eprintln!("You must provide a valid file path");
    }

    println!("Processing done!")
}