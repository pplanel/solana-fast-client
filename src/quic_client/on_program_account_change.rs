use std::fmt::format;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use futures_util::StreamExt;
use napi::bindgen_prelude::{BigInt, FromNapiRef, FromNapiValue};
use napi::threadsafe_function::ErrorStrategy::CalleeHandled;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::{Env, JsExternal, JsNumber, JsObject, JsString, NapiValue, Ref, Result};
use serde::{Deserialize, Serialize};
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::nonblocking::tpu_client::TpuClient;
use solana_client::nonblocking::{self, pubsub_client};
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::RpcFilterType;
use solana_client::tpu_client::TpuClientConfig;
use solana_program::native_token::sol_to_lamports;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_quic_client::{QuicConfig, QuicConnectionManager, QuicPool};
use solana_sdk::account::Account;
use solana_sdk::account_info;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use tokio::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct ProgramAccountsInfoConfig {
  pub filters: Option<Vec<RpcFilterType>>,
  pub account_config: RpcProgramAccountsConfig,
  pub with_context: Option<bool>,
}
