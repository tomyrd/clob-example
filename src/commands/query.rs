use clap::Parser;

use miden_client::{
    auth::TransactionAuthenticator, crypto::FeltRng, rpc::NodeRpcClient, store::Store, Client,
};

use super::sync::SyncCmd;

#[derive(Debug, Clone, Parser)]
#[clap(about = "Query rollup for notes with a certain tag")]
pub struct QueryCmd {
    /// Tag to be queried from the rollup
    tag: u32,
}

impl QueryCmd {
    pub async fn execute<N: NodeRpcClient, R: FeltRng, S: Store, A: TransactionAuthenticator>(
        &self,
        mut client: Client<N, R, S, A>,
    ) -> Result<(), String> {
        client
            .add_note_tag(self.tag.into())
            .map_err(|e| e.to_string())?;

        // Sync rollup state
        let sync_command = SyncCmd {};
        sync_command.execute(client).await?;

        Ok(())
    }
}
