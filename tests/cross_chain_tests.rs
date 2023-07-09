#![cfg(not(target_arch = "wasm32"))]

use async_graphql::InputType;
use linera_sdk::base::{Amount, ApplicationId, Owner};
use linera_sdk::test::{ActiveChain, TestValidator};
use fungible::{Account, FungibleAbi, Operation};

#[tokio::test]
async fn test_cross_chain_transfer() {
    let initial_amount = Amount::from(1_000_000u128);
    let transfer_amount = Amount::from(50_000u128);

    let (validator, bytecode_id) = TestValidator::with_current_bytecode().await;
    let mut sender_chain = validator.new_chain().await;
    let sender_account = Owner::from(sender_chain.public_key());

    let application_id = sender_chain
        .create_application::<fungible::FungibleAbi>(
            bytecode_id,
            (),
            initial_amount,
            vec![]
        ).await;

    let receiver_chain = validator.new_chain().await;
    let receiver_account = Owner::from(receiver_chain.public_key());

    sender_chain.add_block(|block| {
        block.with_operation(
            application_id,
            Operation::Transfer {
                owner: sender_account,
                amount: transfer_amount,
                target_account: Account {
                    chain_id: receiver_chain.id(),
                    owner: receiver_account
                }
            },
        );
    }).await;

    assert_eq!(
        query_account(application_id, &sender_chain, sender_account).await,
        Some(initial_amount.saturating_sub(transfer_amount))
    );

    receiver_chain.handle_received_messages().await;

    assert_eq!(
        query_account(application_id, &receiver_chain, receiver_account).await,
        Some(transfer_amount)
    )

}

async fn query_account(
    application_id: ApplicationId<FungibleAbi>,
    chain: &ActiveChain,
    account_owner: Owner
) -> Option<Amount> {
    let query = format!(
        "query {{ accounts(owner: {}) }}",
        account_owner.to_value()
    );

    let value = chain.graphql_query(application_id, query).await;
    let balance = value.as_object()?.get("accounts")?.as_str()?;

    Some(balance.parse().unwrap())
}