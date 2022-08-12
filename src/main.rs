mod engine;
mod transaction;
mod account;
mod error;

//if true, allows transaction parser to just skip invalid transaction entries in the csv.
//if false, returns an error whenever we find an invalid transaction
static STRICT_PARSE: bool = false;

fn main() {
    let mut engine = engine::PaymentEngine::new();
    let _res = engine.read_csv();
    let _res = engine.process_transactions();
    engine.output_accounts();
}
