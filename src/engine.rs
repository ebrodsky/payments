use std::error::Error;
use std::env;
use csv;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

/*
Assumptions:
Disputes, Resolves and Chargebacks can only refernce a Deposit or Withdrawal with the tx_id.
Clients cannot dispute transactions that are not theirs. In this sense the client field for
disputes, resolves and chargebacks is unecessary, as it's not the client but the system saying
that there is an issue with a specific transaction.
When account is locked, cannot process deposits or withdrawals. Can still process disputes, resolves and chargebacks.
This is because deposits and withdrawals are user-actions, and the other 3 are system actions once something is wrong.
*/

enum TxType{
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
    Unknown
}

#[derive(Debug, Deserialize, Clone)]
/*
    Represents a line in the csv file. 5 types of transactions: Deposit, Withdraw, Dispute, Resolve and Chargeback.
    We separate these into two categories: Transfers and non-transfers. Transfers are deposits and withdrawals,
    non-transfers are everything that references a tx_id, such as dispute, resolve and chargeback.
*/
pub struct Transaction {
    #[serde(rename = "type")]
    tx_type: String,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    tx_id: u32,
    amount: Option<f32>,
    #[serde(skip)]
    dispute: bool
}


impl Transaction {
    fn get_type(&self) -> TxType{
        match self.tx_type.as_str() {
            "deposit"   => TxType::Deposit,
            "withdrawal"=> TxType::Withdrawal,
            "dispute"   => TxType::Dispute,
            "chargeback"=> TxType::Chargeback,
            "resolve"   => TxType::Resolve,
            _           => TxType::Unknown
        }
    }

    //Not specified in instructions, but it's a good idea to make sure the amount is positive for withdrawals and deposits
    fn is_valid_transfer(&self) -> bool {
        let mut valid = false;
        match self.get_type(){
            TxType::Deposit | TxType::Withdrawal => {
                if let Some(v) = self.amount {
                    if v > 0.0{
                        valid = true;
                    }
                }
                valid
            },
            _ => false
        }
    }
}
impl std::fmt::Display for Transaction{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},\t{},\t{},\t{:.4},\t{}", self.tx_type, self.client_id, self.tx_id, self.amount.unwrap_or(0.0), self.dispute)

    }
}

#[derive(Debug, Serialize)]
struct Account {
    #[serde(rename = "client")]
    client_id: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool
}

impl Account {
    //check if withdrawal or deposit and add or remove funds based on that.
    pub fn process_transfer(&mut self, transaction: &Transaction){
        let amount = transaction.amount.unwrap_or(0.0);
        match transaction.get_type(){
            TxType::Deposit => {
                self.available += amount;
                self.total += amount;
            },
            TxType::Withdrawal => {
                if self.available >= amount{ //Make sure client has sufficient funds.
                    self.available -= amount;
                    self.total -= amount;
                }
            },
            _ => {}
        }
    }

    //Marks transaction as disputed and modifies available and held values according to transaction type
    pub fn dispute_transaction(&mut self, transaction: &mut Transaction){
        let amount = transaction.amount.unwrap_or(0.0);
        transaction.dispute = true; //mark the transaction as disputed.
        match transaction.get_type(){
            TxType::Deposit => {
                self.held += amount;
                self.available -= amount;
            },
            TxType::Withdrawal => {
                self.held -= amount;
                self.available += amount;
            }
            _ => panic!("Invalid transaction to accept. Not Deposit or Withdrawal")
        }
    }

    //Unmarks transactions as disputed and mofidies available and held values according to transaction type
    pub fn resolve_transaction(&mut self, transaction: &mut Transaction) {
        let amount = transaction.amount.unwrap_or(0.0);
        transaction.dispute = false;
        match transaction.get_type(){
            TxType::Deposit => {
                self.held -= amount;
                self.available += amount;
            },
            TxType::Withdrawal => {
                self.held += amount;
                self.available -= amount;
            }
            _ => panic!("Invalid transaction to accept. Not Deposit or Withdrawal")

        }
    }
    
    //Reverses a transaction and marks the account as locked.
    pub fn chargeback_transaction(&mut self, transaction: &Transaction) {
        let amount = transaction.amount.unwrap_or(0.0);
        self.locked = true;
        match transaction.get_type(){
            TxType::Deposit => {
                self.held -= amount;
                self.total -= amount;
            },
            TxType::Withdrawal => {
                self.held += amount;
                self.total += amount;
            },
            _ => panic!("Invalid transaction to accept. Not Deposit or Withdrawal")
        }
    }
}
impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},\t{:.4},\t{:.4},\t{:.4},\t{}", self.client_id, self.available, self.held, self.total, self.locked)
    }
}

/*
    struct to keep information about set of transactions and accounts. Can be easily modified to have multiple payment engines running at the same time,
    as each engine can get a csv of its own and work on it independently, assuming transaction order can be given, since entries in each csv are chronological but 
    in order to process multiple csvs at the same time we would need a time stamp field in the csv to ensure chronological order between multiple csvs at the same time.
*/
pub struct PaymentEngine {
    transactions: HashMap<u32, Transaction>,
    accounts: HashMap<u16, Account>,
    entries: Vec<Transaction>
}

impl PaymentEngine {
    pub fn new() -> Self {
        PaymentEngine { transactions: HashMap::new(), accounts: HashMap::new(), entries: vec![]}
    }

    //Read the user-provided csv from the command line arguments.
    pub fn read_csv(&mut self) -> Result<(), Box<dyn Error>> {
        let args: Vec<String> = env::args().collect();
        let csv_name = &args[1];
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
            //println!("{}", entry);
            let client_id = entry.client_id;
            let account = self.accounts.entry(client_id).or_insert(Account{
                client_id,
                available: 0.0,
                held: 0.0,
                total: 0.0,
                locked: false,
            });

            match entry.get_type() {
                TxType::Deposit | TxType::Withdrawal => { //if account is locked, ignore transaction.
                    if entry.is_valid_transfer() && account.locked == false{
                        account.process_transfer(&entry);
                        self.transactions.insert(entry.tx_id, entry.to_owned());
                    }
                },
                TxType::Dispute => {
                    if self.transactions.contains_key(&entry.tx_id){
                        let rel_tx = self.transactions.get_mut(&entry.tx_id).unwrap();
                        if rel_tx.dispute == false{
                            account.dispute_transaction(rel_tx);
                        }

                    }
                },                
                TxType::Resolve => {
                    if self.transactions.contains_key(&entry.tx_id){
                        let rel_tx = self.transactions.get_mut(&entry.tx_id).unwrap();
                        if rel_tx.dispute == true{
                            account.resolve_transaction(rel_tx);
                        }
                    }
                },
                TxType::Chargeback => {
                    if self.transactions.contains_key(&entry.tx_id){
                        let rel_tx = self.transactions.get_mut(&entry.tx_id).unwrap();
                        if rel_tx.dispute == true{
                            account.chargeback_transaction(rel_tx);

                        }
                    }
                },
                TxType::Unknown => panic!("Unknown transaction type.")
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
