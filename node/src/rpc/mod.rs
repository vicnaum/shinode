//! JSON-RPC server.

use eyre::{Result, WrapErr};
use jsonrpsee::{
    server::{ServerBuilder, ServerHandle},
    types::ErrorObjectOwned,
    RpcModule,
};
use serde_json::Value;
use std::net::SocketAddr;

/// Start the JSON-RPC server and return its handle.
pub async fn start(bind: SocketAddr, chain_id: u64) -> Result<ServerHandle> {
    let server = ServerBuilder::default()
        .build(bind)
        .await
        .wrap_err("failed to bind RPC server")?;

    let mut module = RpcModule::new(chain_id);
    module
        .register_method("eth_chainId", |params, chain_id, _| {
            let params: Option<Value> = params.parse()?;
            let empty = match params {
                None | Some(Value::Null) => true,
                Some(Value::Array(values)) => values.is_empty(),
                Some(Value::Object(values)) => values.is_empty(),
                Some(_) => false,
            };
            if !empty {
                return Err(ErrorObjectOwned::owned(
                    -32602,
                    "eth_chainId expects no params",
                    None::<()>,
                ));
            }

            Ok(format!("0x{:x}", *chain_id))
        })
        .wrap_err("failed to register eth_chainId")?;

    Ok(server.start(module))
}
