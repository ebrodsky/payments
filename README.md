Assumptions:
Disputes, Resolves and Chargebacks can only refernce a Deposit or Withdrawal with the tx_id.

Clients cannot dispute transactions that are not theirs. In this sense the client field for disputes, resolves and chargebacks is unecessary, as it's not the client but the system saying that there is an issue with a specific transaction.

When account is locked, cannot process deposits or withdrawals. Can still process disputes, resolves and chargebacks. This is because deposits and withdrawals are user-actions, and the other 3 are system actions once something is wrong.

I assume that since the disputes, resolves and chargebacks are coming from the partner's side, that there is some correctness with the entries. For example, two chargebacks are possible for the same transaction and are processed fine by the program, but of course should not actually happen. This is because after an account is locked after a chargeback, I still allow all non transfer transactions (not deposit or withdrawal) on that account for its transactions.

Disputes on a withdrawal act in an opposite way as on deposits. This means that the held amount for a transaction could be negative. If it's negative, this means that the withdrawal transaction is disputed, and once resolve the held amount will increase and the available amount decrease, representing a successfully resolved withdrawal. 
