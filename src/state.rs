use linera_sdk::base::{Amount, Owner};
use linera_sdk::views::{MapView, ViewStorageContext};
use linera_views::views::{GraphQLView, RootView};
use thiserror::Error;

#[derive(RootView, GraphQLView)]
#[view(context = "ViewStorageContext")]
pub struct FungibleToken {
    accounts: MapView<Owner, Amount>,
}

#[allow(dead_code)]
impl FungibleToken {
    pub async fn initialize_accounts(&mut self, owner: Owner, amount: Amount) {
        log::info!("Initialising {} with {} tokens.", owner, amount);
        self.accounts
            .insert(&owner, amount)
            .expect("Error in insert statemet")
    }

    pub async fn balance(&self, account: &Owner) -> Amount {
        self.accounts
            .get(account)
            .await
            .expect("Failure in retrieval")
            .unwrap_or_default()
    }

    pub async fn credit(&mut self, account: Owner, amount: Amount) {
        log::info!("Owner {} received {} tokens", account, amount);
        let mut balance = self.balance(&account).await;
        balance.saturating_add_assign(amount);
        self.accounts
            .insert(&account, balance)
            .expect("Failed to insert")
    }

    pub async fn debit(
        &mut self,
        account: Owner,
        amount: Amount,
    ) -> Result<(), InsufficientBalanceError> {
        log::info!("Owner {} was debited {} tokens", account, amount);
        let mut balance = self.balance(&account).await;
        balance
            .try_sub_assign(amount)
            .map_err(|_| InsufficientBalanceError)?;
        self.accounts
            .insert(&account, balance)
            .expect("Failed to insert");
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Error)]
#[error("Insufficient balance error")]
pub struct InsufficientBalanceError;
