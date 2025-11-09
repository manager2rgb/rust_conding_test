mod client;
mod payments_engine;
mod transaction;

use clap::{Arg, ArgAction, Command};

use payments_engine::PaymentsEngine;

use transaction::Transaction;

async fn start_transactions_service(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = filename.trim();

    let metadata_file = std::fs::OpenOptions::new().read(true).open(path)?;
    let buffered = std::io::BufReader::new(metadata_file);

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) //Whitespaces and decimal precisions (up to four places past the decimal) must be accepted by your program.
        .delimiter(b',')
        .from_reader(buffered);

    let mut iter = rdr.deserialize();

    let mut payments_engine = PaymentsEngine::new();

    while let Some(transaction_result) = iter.next() {
        let transaction: Transaction = transaction_result.unwrap();
        payments_engine.handle_transaction(transaction).await;
    }
    payments_engine.write_state().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Command::new("Payments Engine");
    parser = parser.arg(
        Arg::new("file")
            .display_order(1)
            .alias("metadata")
            .help("Provide transtactions.csv file")
            .action(ArgAction::Set)
            .value_name("TRANSACTIONS_FILE.csv")
            .value_parser(clap::builder::NonEmptyStringValueParser::new())
            .required(true),
    );

    let args = parser.get_matches();

    let filename = args.get_one::<String>("file").unwrap();

    start_transactions_service(filename).await?;

    Ok(())
}
