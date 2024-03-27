use std::ops::Deref;
use std::sync::Arc;

use napi::threadsafe_function::ErrorStrategy::CalleeHandled;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Result;

use futures_util::StreamExt;
use napi_derive::napi;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::nonblocking::tpu_client::TpuClient;
use solana_client::nonblocking::{self, pubsub_client};
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::tpu_client::TpuClientConfig;
use solana_program::pubkey::Pubkey;
use solana_quic_client::{QuicConfig, QuicConnectionManager, QuicPool};
use solana_sdk::transaction::Transaction;

#[napi(js_name = "QuicClient")]
pub struct QuicClient {
  rpc_url: String,
  ws_url: String,
  tpu_client: Option<TpuClient<QuicPool, QuicConnectionManager, QuicConfig>>,
  pub_sub_client: Arc<Option<pubsub_client::PubsubClient>>,
}

impl Deref for QuicClient {
  type Target = TpuClient<QuicPool, QuicConnectionManager, QuicConfig>;

  fn deref(&self) -> &Self::Target {
    self.tpu_client.as_ref().unwrap()
  }
}

#[napi]
impl QuicClient {
  #[napi(factory)]
  pub fn new(rpc_url: String, ws_url: String) -> Self {
    Self {
      tpu_client: None,
      pub_sub_client: Arc::new(None),
      rpc_url,
      ws_url,
    }
  }

  #[napi]
  pub async unsafe fn connect(&mut self) -> Result<()> {
    self.tpu_client = Some(create_tpu_client(&self.rpc_url, &self.ws_url).await?);
    self.pub_sub_client = Arc::new(Some(create_pub_sub_client(&self.ws_url).await?));
    Ok(())
  }

  #[napi]
  pub async fn send_transaction(&self, tx_slice: &[u8]) -> Result<String> {
    let tx: Transaction = bincode::deserialize(tx_slice).unwrap();

    self
      .tpu_client
      .as_ref()
      .unwrap()
      .try_send_transaction(&tx)
      .await
      .map_err(|_| napi::Error::new(napi::Status::Unknown, "deserialize transaction"))?;

    Ok(tx.verify_and_hash_message().unwrap().to_string())
  }

  #[napi]
  pub async fn on_program_account_change(
    &self,
    pubkey: String,
    config: serde_json::Value,
    tsfn: ThreadsafeFunction<String, CalleeHandled>,
  ) -> napi::Result<()> {
    let tsfn = tsfn.clone();
    let pk: Pubkey = pubkey.parse().unwrap();
    let mut config: RpcProgramAccountsConfig = serde_json::from_value(config).unwrap();
    config.account_config.encoding = Some(UiAccountEncoding::Base64);

    let p_client = nonblocking::pubsub_client::PubsubClient::new(&self.ws_url)
      .await
      .unwrap();

    let client = Arc::new(p_client);
    tokio::task::spawn(async move {
      if let Ok((mut notifications, _)) = client.program_subscribe(&pk, Some(config)).await {
        while let Some(account_info_response) = notifications.next().await {
          let account_info = account_info_response.value;

          tsfn.call(
            Ok(serde_json::to_string(&account_info).unwrap()),
            napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
          );
        }
      };
    });

    Ok(())
  }
}

async fn create_pub_sub_client(ws_url: &str) -> napi::Result<PubsubClient> {
  pubsub_client::PubsubClient::new(ws_url)
    .await
    .map_err(|err| napi::Error::new(napi::Status::Unknown, err.to_string()))
}

async fn create_tpu_client(
  rpc_url: &str,
  ws_url: &str,
) -> napi::Result<TpuClient<QuicPool, QuicConnectionManager, QuicConfig>> {
  let rpc_client = RpcClient::new(rpc_url.to_string());

  TpuClient::<QuicPool, QuicConnectionManager, QuicConfig>::new(
    "quick_client",
    Arc::from(rpc_client),
    ws_url,
    TpuClientConfig::default(),
  )
  .await
  .map_err(|e| {
    napi::Error::new(
      napi::Status::Unknown,
      format!("QuicClient::connect error: {e:?}"),
    )
  })
}

#[tokio::test]
async fn test_on_program_account_change() {
  use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
  use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
  pub async fn on_program_account_change(pubkey: String) -> napi::Result<()> {
    let pk: Pubkey = pubkey.parse().unwrap();
    let config: RpcProgramAccountsConfig = RpcProgramAccountsConfig {
      filters: None,
      account_config: RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        data_slice: None,
        commitment: Some(CommitmentConfig {
          commitment: CommitmentLevel::Finalized,
        }),
        min_context_slot: Some(0),
      },
      with_context: None,
    };

    let p_client =
      nonblocking::pubsub_client::PubsubClient::new("wss://api.mainnet-beta.solana.com")
        .await
        .unwrap();

    let client = Arc::new(p_client);
    let h = tokio::task::spawn(async move {
      if let Ok((mut notifications, _)) = client.program_subscribe(&pk, Some(config)).await {
        while let Some(account_info_response) = notifications.next().await {
          let account_info = account_info_response.value;

          dbg!(account_info);
        }
      }
    });

    let _ = h.await;

    Ok(())
  }

  let _ =
    on_program_account_change(String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")).await;
}
