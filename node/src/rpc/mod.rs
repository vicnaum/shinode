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

    Ok(server.start(module(chain_id)?))
}

pub fn module(chain_id: u64) -> Result<RpcModule<u64>> {
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

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;
    use jsonrpsee::rpc_params;

    async fn start_test_server(chain_id: u64) -> SocketAddr {
        let server = ServerBuilder::default()
            .build("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = server.local_addr().expect("local addr");
        let handle = server.start(module(chain_id).expect("module"));
        tokio::spawn(handle.stopped());
        addr
    }

    #[tokio::test]
    async fn eth_chain_id_success() {
        let addr = start_test_server(1).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let result: String = client
            .request("eth_chainId", rpc_params![])
            .await
            .expect("chain id");
        assert_eq!(result, "0x1");
    }

    #[tokio::test]
    async fn eth_chain_id_rejects_params() {
        let addr = start_test_server(1).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let err = client
            .request::<String, _>("eth_chainId", rpc_params![1])
            .await
            .expect_err("invalid params");
        match err {
            jsonrpsee::core::ClientError::Call(err_obj) => {
                assert_eq!(err_obj.code(), -32602);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn unknown_method_returns_method_not_found() {
        let addr = start_test_server(1).await;
        let client = HttpClientBuilder::default()
            .build(&format!("http://{addr}"))
            .expect("client");
        let err = client
            .request::<String, _>("eth_blockNumber", rpc_params![])
            .await
            .expect_err("method not found");
        match err {
            jsonrpsee::core::ClientError::Call(err_obj) => {
                assert_eq!(err_obj.code(), -32601);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
