#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::FungibleToken;
use crate::state::InsufficientBalanceError;
use async_trait::async_trait;
use fungible::{Account, Message, Operation};
use linera_sdk::base::{Amount, Owner};
use linera_sdk::contract::system_api;
use linera_sdk::{
    base::{SessionId, WithContractAbi},
    ApplicationCallResult, CalleeContext, Contract, ExecutionResult, MessageContext,
    OperationContext, SessionCallResult, ViewStateStorage,
};
use thiserror::Error;

linera_sdk::contract!(FungibleToken);

impl WithContractAbi for FungibleToken {
    type Abi = fungible::FungibleAbi;
}

#[async_trait]
impl Contract for FungibleToken {
    type Error = Error;
    type Storage = ViewStateStorage<Self>;

    async fn initialize(
        &mut self,
        _context: &OperationContext,
        argument: Self::InitializationArgument,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        if let Some(owner) = _context.authenticated_signer {
            self.initialize_accounts(owner, argument).await
        }
        Ok(ExecutionResult::default())
    }

    async fn execute_operation(
        &mut self,
        context: &OperationContext,
        operation: Self::Operation,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        match operation {
            Operation::Transfer {
                owner,
                amount,
                target_account,
            } => {
                Self::check_account_authentication(context.authenticated_signer, owner)?;
                self.debit(owner, amount).await?;
                Ok(self
                    .finish_transfer_to_account(amount, target_account)
                    .await)
            }
        }
    }

    async fn execute_message(
        &mut self,
        _context: &MessageContext,
        message: Self::Message,
    ) -> Result<ExecutionResult<Self::Message>, Self::Error> {
        match message {
            Message::Credit { amount, owner } => {
                self.credit(owner, amount).await;
                Ok(ExecutionResult::default())
            }
        }
    }

    async fn handle_application_call(
        &mut self,
        _context: &CalleeContext,
        _call: Self::ApplicationCall,
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<ApplicationCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error>
    {
        Ok(ApplicationCallResult::default())
    }

    async fn handle_session_call(
        &mut self,
        _context: &CalleeContext,
        _session: Self::SessionState,
        _call: Self::SessionCall,
        _forwarded_sessions: Vec<SessionId>,
    ) -> Result<SessionCallResult<Self::Message, Self::Response, Self::SessionState>, Self::Error>
    {
        Err(Error::SessionsNotSupported)
    }
}

#[allow(dead_code)]
impl FungibleToken {
    fn check_account_authentication(
        authenticated_signed: Option<Owner>,
        owner: Owner,
    ) -> Result<(), Error> {
        if authenticated_signed == Some(owner) {
            return Ok(());
        }
        Err(Error::IncorrectAuthentication)
    }

    async fn finish_transfer_to_account(
        &mut self,
        amount: Amount,
        account: Account,
    ) -> ExecutionResult<Message> {
        if account.chain_id == system_api::current_chain_id() {
            self.credit(account.owner, amount).await;
            ExecutionResult::default()
        } else {
            let message = Message::Credit {
                owner: account.owner,
                amount: amount,
            };
            ExecutionResult::default().with_message(account.chain_id, message)
        }
    }
}

/// An error that can occur during the contract execution.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to deserialize BCS bytes
    #[error("Failed to deserialize BCS bytes")]
    BcsError(#[from] bcs::Error),

    /// Failed to deserialize JSON string
    #[error("Failed to deserialize JSON string")]
    JsonError(#[from] serde_json::Error),

    #[error("Incorrect Authentication")]
    IncorrectAuthentication, // Add more error variants here.

    #[error("Insufficient Balance")]
    InsufficientBalance(#[from] InsufficientBalanceError),

    #[error("Sessions not supported")]
    SessionsNotSupported,
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use futures::FutureExt;
    use linera_sdk::base::{BlockHeight, ChainId};
    use linera_sdk::test;
    use linera_sdk::views::crypto::KeyPair;
    use linera_views::crypto::PublicKey;

    use webassembly_test::webassembly_test;

    #[webassembly_test]
    pub fn initialisation() {
        let fungible = create_and_initialize_fungible(initial_amount());
        assert_eq!(
            fungible.balance(&creator()).now_or_never().unwrap(),
            initial_amount()
        )
    }

    #[webassembly_test]
    pub fn transfer() {
        let mut fungible = create_and_initialize_fungible(initial_amount());
        test::mock_chain_id(ChainId::root(1));
        
        let transfer = Operation::Transfer {
            owner: creator(),
            amount: Amount::from(50u128),
            target_account: Account {
                chain_id: ChainId::root(1),
                owner: creator(),  // FIXME
            },
        };

        let result = fungible
            .execute_operation(&dummy_operation_context(), transfer)
            .now_or_never()
            .expect("Initialization of counter state should not await anything");

        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), ExecutionResult::default());
    }

    fn create_and_initialize_fungible(initial_value: Amount) -> FungibleToken {
        let mut fungible = FungibleToken::dummy();

        let result = fungible
            .initialize(&dummy_operation_context(), initial_value)
            .now_or_never()
            .expect("Initialization of counter state should not await anything");

        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), ExecutionResult::default());

        fungible
    }

    fn dummy_operation_context() -> OperationContext {
        OperationContext {
            chain_id: ChainId([0; 4].into()),
            authenticated_signer: Some(creator()),
            height: BlockHeight(0),
            index: 0,
        }
    }

    fn creator() -> Owner {
        "1c02a28d03e846b113de238d8880df3c9c802143b73aea5d173466701bee1786"
            .parse()
            .unwrap()
    }

    fn initial_amount() -> Amount {
        Amount::from(1_000_000u128)
    }
}
