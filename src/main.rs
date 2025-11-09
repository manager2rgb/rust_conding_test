mod client;
mod payments_engine;
mod transaction;
mod types;

use clap::{Arg, ArgAction, Command};

use payments_engine::PaymentsEngine;

fn start_transactions_service(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = filename.trim();

    let metadata_file = std::fs::OpenOptions::new().read(true).open(path)?;
    let buffered = std::io::BufReader::new(metadata_file);

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) //Whitespaces must be accepted
        .delimiter(b',')
        .flexible(true)
        .from_reader(buffered);

    let iter = rdr.deserialize();

    let mut payments_engine = PaymentsEngine::new();

    for transaction_result in iter {
        match transaction_result {
            Ok(transaction) => payments_engine.handle_transaction(transaction),
            Err(e) => eprintln!("Failed to parse transaction: {:?}", e),
        }
    }
    let output = payments_engine.write_state();
    print!("{}", output);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    start_transactions_service(filename)?;

    Ok(())
}
