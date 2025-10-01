use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use aura_chain_runtime::{opaque::Block, AccountId, Balance, Index};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait AuraIdentityRpc<BlockHash> {
    #[rpc(name = "auraidentity_getIdentity")]
    fn get_identity(&self, account: AccountId, at: Option<BlockHash>) -> Result<Option<String>>;
}

pub struct AuraIdentityRpcImpl<C> {
    client: Arc<C>,
}

impl<C> AuraIdentityRpcImpl<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C> AuraIdentityRpc<<Block as BlockT>::Hash> for AuraIdentityRpcImpl<C>
where
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: pallet_aura_identity_runtime_api::AuraIdentityRuntimeApi<Block, AccountId>,
{
    fn get_identity(&self, account: AccountId, at: Option<<Block as BlockT>::Hash>) -> Result<Option<String>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash
        ));

        api.get_identity(&at, account)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(9876),
                message: "Unable to query identity".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }
}

// RPC extensions container
pub struct FullDeps<C> {
    pub client: Arc<C>,
    pub pool: Arc<sc_transaction_pool::FullPool<Block, C>>,
    pub deny_unsafe: sc_rpc::DenyUnsafe,
}

pub fn create_full<C>(
    deps: FullDeps<C>,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + Send
        + Sync
        + 'static
        + sc_client_api::BlockBackend<Block>,
    C::Api: pallet_aura_identity_runtime_api::AuraIdentityRuntimeApi<Block, AccountId>,
{
    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        deny_unsafe,
    } = deps;

    io.extend_with(
        crate::rpc::AuraIdentityRpc::to_delegate(AuraIdentityRpcImpl::new(client.clone()))
    );

    io
}
