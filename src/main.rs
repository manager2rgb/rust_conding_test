mod client;
mod engine;
mod storage;
mod transaction;
mod types;

use clap::{Arg, ArgAction, Command};
use tokio::task::JoinSet;

use crate::engine::payments_engine::PaymentsEngine;

async fn start_transactions_service(
    payments_engine: PaymentsEngine,
    filename: String,
) -> Result<(), ()> {
    let path = filename.trim();

    let metadata_file = std::fs::OpenOptions::new().read(true).open(path).unwrap();
    let buffered = std::io::BufReader::new(metadata_file);

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) //Whitespaces must be accepted
        .delimiter(b',')
        .flexible(true)
        .from_reader(buffered);

    let iter = rdr.deserialize();

    for transaction_result in iter {
        match transaction_result {
            Ok(transaction) => match payments_engine.handle_transaction(transaction).await {
                Ok(_) => {}
                Err(_err) => {
                    //eprintln!("Engine error : {}", err);
                }
            },
            Err(err) => eprintln!("Error deserializing transaction: {}", err),
        }
    }
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

    let filename = args.get_one::<String>("file").unwrap().clone();

    let payments_engine = PaymentsEngine::new();

    let mut set = JoinSet::new();
    set.spawn(start_transactions_service(
        payments_engine.clone(),
        filename,
    ));
    // set.spawn(start_transactions_service(
    //     payments_engine.clone(),
    //     "transactions_large.csv".to_string(),
    // )); // //Just to test if it work as expected

    set.join_all().await;

    match payments_engine.write_state().await {
        Ok(output) => {
            print!("{}", output);
        }
        Err(err) => {
            eprintln!("Engine error : {}", err);
        }
    }

    Ok(())
}
