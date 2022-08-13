use std::env;

mod engine;
mod transaction;
mod account;
mod error;

//if true, allows transaction parser to just skip invalid transaction entries in the csv.
//if false, returns an error whenever we find an invalid transaction
static STRICT_PARSE: bool = true;

fn main() {
    let args: Vec<String> = env::args().collect();
    let csv_name = &args[1];
    let mut engine = engine::PaymentEngine::new();
    let _res = engine.read_csv(csv_name);
    let _res = engine.process_transactions();
    engine.output_accounts();
}


#[cfg(test)]
#[allow(unused_assignments)]
mod tests{
    use csv::WriterBuilder;
    use std::error::Error;
    use crate::transaction::TxType;
    use rand::Rng;

    use crate::engine::PaymentEngine;
    use crate::account::Account;

    fn transaction(tx_type: &TxType, client_id: u16, tx_id: u32, amount: Option<f32>) -> String {
        let input;

        match tx_type{
            TxType::Deposit | TxType::Withdrawal => {
                input = format!("{}, {}, {}, {}", tx_type.to_string(), client_id, tx_id, amount.unwrap()).to_owned();
            },
            TxType::Dispute | TxType::Resolve | TxType::Chargeback => {
                input = format!("{}, {}, {},", tx_type.to_string(), client_id, tx_id).to_owned();
            },

            TxType::Unknown => unreachable!(),
        }
        input
        
    }
    //dep neg amount
    //with neg amount
    //withdraw more than available
    //dispute failed withdrawal (should be same as disputing tx that does not exist)
    //dispute a tx that does not exist
    //resolve a tx under dispute
    //resolve a tx not under dispute
    //chargeback a tx under dispute
    //chargeback a tx not under dispute
    //deposit to a locked account
    //withdraw from a locked account
    //resolve dispute with a locked account
    //chargeback a tx from a locked account
    #[test]
    fn test_random() -> Result<(), Box<dyn Error>>{
        let mut csv_writer = WriterBuilder::new().flexible(true).from_path("test_basic.csv")?;
        csv_writer.write_record(&["type", "client", "tx", "amount"])?;

        let n_transactions = 5;
        let n_accounts = 3;
        let mut entries:Vec<String> = vec![];
        let mut resolves:Vec<String> = vec![];
        let mut rng = rand::thread_rng();
        let tx_types = [TxType::Deposit, TxType::Withdrawal, TxType::Dispute, TxType::Resolve, TxType::Chargeback];

        let mut tx_type = &TxType::Unknown;
        let mut client_id = 0;
        let mut tx_id = 0;
        let mut amount = 0.0;

        let mut i = 1;
        while i < n_transactions{
            tx_type = &tx_types[rng.gen_range(0..5)];
            client_id = rng.gen_range(1..n_accounts);
            match tx_type{
                TxType::Deposit | TxType::Withdrawal => {
                    tx_id = i;
                    i += 1;
                },
                TxType::Dispute | TxType::Resolve | TxType::Chargeback=> {
                    tx_id = rng.gen_range(1..i+1);
                },
                TxType::Unknown => unreachable!(),
            }
            amount = rng.gen_range(1..100) as f32;
            let tx = transaction(&tx_type, client_id, tx_id, Some(amount));
            entries.push(tx);
            resolves.push(transaction(&TxType::Resolve, client_id, tx_id, Some(amount)));
        }
        for entry in &entries{
            csv_writer.write_record(entry.split(",").collect::<Vec<&str>>())?;
        }

        for entry in &resolves{
            csv_writer.write_record(entry.split(",").collect::<Vec<&str>>())?;
        }
        csv_writer.flush()?;
        let mut engine = PaymentEngine::new();
        let _res = engine.read_csv(&"test_basic.csv".to_string());
        let _res = engine.process_transactions();
        let accounts = engine.get_accounts();
        for (_id, account) in accounts{
            assert!(account.get_available() >= 0.0);
        }

        Ok(())
    }
}