use async_graphql::{InputObject, Request, Response};
use linera_sdk::base::{Amount, ChainId, ContractAbi, Owner, ServiceAbi};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct FungibleAbi;

impl ContractAbi for FungibleAbi {
    type Parameters = ();
    type InitializationArgument = Amount;
    type Operation = Operation;
    type Message = Message;
    type ApplicationCall = ();
    type SessionCall = ();
    type SessionState = ();
    type Response = ();
}

impl ServiceAbi for FungibleAbi {
    type Parameters = ();
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    Transfer {
        owner: Owner,
        amount: Amount,
        target_account: Account,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    Credit { amount: Amount, owner: Owner },
}

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, InputObject,
)]
pub struct Account {
    pub chain_id: ChainId,
    pub owner: Owner,
}
