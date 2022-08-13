use crate::STRICT_PARSE;
use crate::transaction::{Transaction, TxType};
use crate::account::Account;
use crate::error::UnknownTransactionError;

use std::error::Error;
use csv;
use std::collections::HashMap;

/*
    struct to keep information about set of transactions and accounts. Can be easily modified to have multiple payment engines running at the same time,
    as each engine can get a csv of its own and work on it independently, assuming transaction order can be given, since entries in each csv are chronological but 
    in order to process multiple csvs at the same time we would need a time stamp field in the csv to ensure chronological order between multiple csvs at the same time.
*/
pub struct PaymentEngine {
    transactions: HashMap<u32, Transaction>,
    accounts: HashMap<u16, Account>,
    entries: Vec<Transaction>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        PaymentEngine { transactions: HashMap::new(), accounts: HashMap::new(), entries: vec![]}
    }
    pub fn get_accounts(&self) -> &HashMap<u16, Account>{
        return &self.accounts;
    }
    //Read the user-provided csv from the command line arguments.
    pub fn read_csv(&mut self, csv_name: &String) -> Result<(), Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b',')
            .trim(csv::Trim::All).from_path(csv_name)?;
            
        for result in reader.deserialize::<Transaction>(){
            let entry = result?;
            self.entries.push(entry.clone());
        }
        Ok(())
    }

    //Output account information for all accounts
    pub fn output_accounts(&self) {
        println!("client,\tavailable,\theld,\ttotal,\tlocked");
        for (_id, account) in &self.accounts {
            println!("{}", account);
        }
    }

    /*
    Process the entries that the payment engine has. This populates a hashmap that stores transfer transactions (Deposit and Withdrawal) (key = tx id)
    Also populates account hashmap that stores account information (key = client id)
    For the entries given to the engine, it processes them one by one until there is none left.
    */
    pub fn process_transactions(&mut self) -> Result<(), Box<dyn Error>> { //TODO change to process_entries    
        for entry in &self.entries{
            let client_id = entry.get_client_id();
            match entry.get_type() {
                TxType::Deposit | TxType::Withdrawal => { //if account is locked, ignore transaction.
                    let account = self.accounts.entry(client_id).or_insert(Default::default());
                    account.set_id(client_id);
                    if entry.is_valid_transfer() && account.is_locked() == false{ //make sure account isn't locked and transfer is valid
                        if let Ok(()) = account.process_transfer(&entry){ //check if client has sufficient funds to withdraw
                            self.transactions.insert(entry.get_tx_id(), entry.to_owned()); //only add transaction if sufficient funds
                        }
                    }
                },
                TxType::Dispute => {
                    let account = self.accounts.entry(client_id).or_insert(Default::default());
                    account.set_id(client_id);
                    if self.transactions.contains_key(&entry.get_tx_id()){ //only act if the transaction exists. 
                        let rel_tx = self.transactions.get_mut(&entry.get_tx_id()).unwrap();
                        if rel_tx.is_disputed() == false && client_id == rel_tx.get_client_id(){ //make sure the only the client who made the tx can act
                            account.dispute_transaction(rel_tx)?
                        }

                    }
                },                
                TxType::Resolve => {
                    let account = self.accounts.entry(client_id).or_insert(Default::default());
                    account.set_id(client_id);
                    if self.transactions.contains_key(&entry.get_tx_id()){ //only act if the transaction exists.
                        let rel_tx = self.transactions.get_mut(&entry.get_tx_id()).unwrap();
                        if rel_tx.is_disputed() == true && client_id == rel_tx.get_client_id(){ //make sure the only the client who made the tx can act
                            account.resolve_transaction(rel_tx)?
                        }
                    }
                },
                TxType::Chargeback => {
                    let account = self.accounts.entry(client_id).or_insert(Default::default());
                    account.set_id(client_id);
                    if self.transactions.contains_key(&entry.get_tx_id()){ //only act if the transaction exists.
                        let rel_tx = self.transactions.get_mut(&entry.get_tx_id()).unwrap();
                        if rel_tx.is_disputed() == true && client_id == rel_tx.get_client_id(){ //make sure the only the client who made the tx can act
                            account.chargeback_transaction(rel_tx)?

                        }
                    }
                },
                TxType::Unknown => { //Check how to deal with unknown types based on strict or loose csv parsing
                    if STRICT_PARSE == true{
                        Err(Box::new(UnknownTransactionError)).unwrap()
                    }
                    else{
                        continue
                    }
                }
            }
            //self.output_accounts();
        }
        self.empty_entries();
        
        Ok(())
    }

    //clear the entries once done with current set. Free up space to allow for more entries to enter with another csv 
    //or a portion of the same csv, depending on size.
    pub fn empty_entries(&mut self){
        self.entries.clear();
    }
}
