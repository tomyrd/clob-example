use crate::{
    order::{sort_orders, Order},
    utils::{get_notes_by_tag, print_order_table},
};
use clap::Parser;
use miden_client::{
    auth::TransactionAuthenticator, crypto::FeltRng, rpc::NodeRpcClient, store::Store, Client,
};

#[derive(Debug, Clone, Parser)]
#[clap(about = "Create a new account and login")]
pub struct ListCmd {
    // Swap tag
    swap_tag: u32,
}

impl ListCmd {
    pub fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        let notes = get_notes_by_tag(&client, self.swap_tag.into());
        let orders: Vec<Order> = notes.into_iter().map(Order::from).collect();

        let sorted_orders = sort_orders(orders);
        let title = format!("Relevant orders for tag {}:", self.swap_tag);
        print_order_table(title.as_str(), &sorted_orders);

        Ok(())
    }
}
