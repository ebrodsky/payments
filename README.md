Assumptions:
Disputes, Resolves and Chargebacks can only refernce a Deposit or Withdrawal with the tx_id.
Clients cannot dispute transactions that are not theirs. In this sense the client field for
disputes, resolves and chargebacks is unecessary, as it's not the client but the system saying
that there is an issue with a specific transaction.
When account is locked, cannot process deposits or withdrawals. Can still process disputes, resolves and chargebacks.
This is because deposits and withdrawals are user-actions, and the other 3 are system actions once something is wrong.
