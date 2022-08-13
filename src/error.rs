use std::{fmt, error::Error};

#[derive(Debug, Clone)]
pub struct NonActionableTransactionError;
impl Error for NonActionableTransactionError{}
impl fmt::Display for NonActionableTransactionError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "References transaction cannot not be referrenced")
    }
}

#[derive(Debug, Clone)]
pub struct UnknownTransactionError;
impl Error for UnknownTransactionError{}
impl fmt::Display for UnknownTransactionError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "References transaction cannot not be referrenced")
    }
}

#[derive(Debug, Clone)]
pub struct InsufficientFundsError;
impl Error for InsufficientFundsError{}
impl fmt::Display for InsufficientFundsError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "References transaction cannot not be referrenced")
    }
}