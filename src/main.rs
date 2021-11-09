use std::env;

mod exchange;

fn main() {
    let mut exchange = exchange::Exchange::new();

    if let Some(file) = env::args().nth(1) {
        if let Err(e) = exchange::process_transactions_from_csv(&file, &mut exchange) {
            eprintln!("Failed to read CSV with exception: {}", e)
        }
        exchange.to_csv();
    } else {
        eprintln!("You must provide a valid file path");
    }
}