use serde::Deserialize;

/*
Assumptions:
Disputes, Resolves and Chargebacks can only refernce a Deposit or Withdrawal with the tx_id.
Clients cannot dispute transactions that are not theirs. In this sense the client field for
disputes, resolves and chargebacks is unecessary, as it's not the client but the system saying
that there is an issue with a specific transaction.
When account is locked, cannot process deposits or withdrawals. Can still process disputes, resolves and chargebacks.
This is because deposits and withdrawals are user-actions, and the other 3 are system actions once something is wrong.
*/

pub enum TxType{
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

    pub fn get_type(&self) -> TxType{
        match self.tx_type.as_str() {
            "deposit"   => TxType::Deposit,
            "withdrawal"=> TxType::Withdrawal,
            "dispute"   => TxType::Dispute,
            "chargeback"=> TxType::Chargeback,
            "resolve"   => TxType::Resolve,
            _           => TxType::Unknown
        }
    }

    pub fn get_amount(&self) -> &Option<f32>{
        return &self.amount;
    }

    pub fn is_disputed(&self) -> bool{
        return self.dispute;
    }

    pub fn set_dispute(&mut self, dispute: bool) {
        self.dispute = dispute;
    }

    pub fn get_client_id(&self) -> u16 {
        return self.client_id;
    }

    pub fn get_tx_id(&self) -> u32{
        return self.tx_id;
    }

    //Not specified in instructions, but it's a good idea to make sure the amount is positive for withdrawals and deposits
    pub fn is_valid_transfer(&self) -> bool {
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