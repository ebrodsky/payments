use std::error::Error;

use crate::transaction::{Transaction, TxType};
use crate::error::{NonActionableTransactionError, InsufficientFundsError};

#[derive(Default, Debug)]
pub struct Account {
    client_id: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool
}

impl Account {
    
    pub fn is_locked(&self) -> bool{
        return self.locked;
    }
    pub fn set_id(&mut self, client_id: u16) {
        self.client_id = client_id;
    }
    #[allow(dead_code)]
    pub fn get_available(&self) -> f32{
        return self.available;
    }

    //check if withdrawal or deposit and add or remove funds based on that.
    pub fn process_transfer(&mut self, transaction: &Transaction) -> Result<(), Box<dyn Error>>{
        let amount = transaction.get_amount().unwrap_or(0.0);
        match transaction.get_type(){
            TxType::Deposit => {
                self.available += amount;
                self.total += amount;
                Ok(())
            },
            TxType::Withdrawal => {
                if self.available >= amount{ //Make sure client has sufficient funds.
                    self.available -= amount;
                    self.total -= amount;
                    Ok(())
                }
                else{
                    Err(Box::new(InsufficientFundsError))
                }
            },
            _ => unreachable!()
        }
    }

    //Marks transaction as disputed and modifies available and held values according to transaction type
    pub fn dispute_transaction(&mut self, transaction: &mut Transaction) -> Result<(), Box<dyn Error>>{
        let amount = transaction.get_amount().unwrap_or(0.0);
        transaction.set_dispute(true); //mark the transaction as disputed.
        match transaction.get_type(){
            TxType::Deposit => {
                self.held += amount;
                self.available -= amount;
                Ok(())
            },
            TxType::Withdrawal => {
                self.held -= amount;
                self.available += amount;
                Ok(())
            }
            _ => Err(Box::new(NonActionableTransactionError)) //Could also be unreachable!()
        }
    }

    //Unmarks transactions as disputed and mofidies available and held values according to transaction type
    pub fn resolve_transaction(&mut self, transaction: &mut Transaction) -> Result<(), Box<dyn Error>>{
        let amount = transaction.get_amount().unwrap_or(0.0);
        transaction.set_dispute(false);
        match transaction.get_type(){
            TxType::Deposit => {
                self.held -= amount;
                self.available += amount;
                Ok(())
            },
            TxType::Withdrawal => {
                self.held += amount;
                self.available -= amount;
                Ok(())
            }
            _ => Err(Box::new(NonActionableTransactionError)) //Could also be unreachable!()
        }
    }
    
    //Reverses a transaction and marks the account as locked.
    pub fn chargeback_transaction(&mut self, transaction: &mut Transaction) -> Result<(), Box<dyn Error>>{
        let amount = transaction.get_amount().unwrap_or(0.0);
        transaction.set_dispute(false);
        self.locked = true;
        match transaction.get_type(){
            TxType::Deposit => {
                self.held -= amount;
                self.total -= amount;
                Ok(())
            },
            TxType::Withdrawal => {
                self.held += amount;
                self.total += amount;
                Ok(())
            },
            _ => Err(Box::new(NonActionableTransactionError)) //Could also be unreachable!()
        }
    }
}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},\t{:.4},\t{:.4},\t{:.4},\t{}", self.client_id, self.available, self.held, self.total, self.locked)
    }
}