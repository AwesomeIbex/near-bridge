use borsh::{BorshDeserialize, BorshSerialize};

use super::{hash::CryptoHash, merkle::MerklePath, types::Gas, views::LightClientBlockLiteView};

pub enum TransactionOrReceiptId {
	Transaction { hash: CryptoHash, sender: AccountId },
	Receipt { id: CryptoHash, receiver: AccountId },
}

pub struct RpcLightClientExecutionProofResponse {
	/// Proof of execution outcome
	pub outcome_proof: ExecutionOutcomeWithIdView,
	/// Proof of shard execution outcome root
	pub outcome_root_proof: MerklePath,
	/// A light weight representation of block that contains the outcome root
	pub block_header_lite: LightClientBlockLiteView,
	/// Proof of the existence of the block in the block merkle tree,
	/// which consists of blocks up to the light client head
	pub block_proof: MerklePath,
}

pub struct ExecutionOutcomeWithIdView {
	/// Proof of the execution outcome
	pub proof: MerklePath,
	/// Block hash of the block that contains the outcome root
	pub block_hash: CryptoHash,
	/// Id of the execution (transaction or receipt)
	pub id: CryptoHash,
	/// The actual outcome
	pub outcome: ExecutionOutcomeView,
}

#[derive(
	BorshSerialize,
	BorshDeserialize,
	Debug,
	Clone,
	PartialEq,
	Eq,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct ExecutionOutcomeView {
	/// Logs from this transaction or receipt.
	pub logs: Vec<String>,
	/// Receipt IDs generated by this transaction or receipt.
	pub receipt_ids: Vec<CryptoHash>,
	/// The amount of the gas burnt by the given transaction or receipt.
	pub gas_burnt: Gas,
	/// The amount of tokens burnt corresponding to the burnt gas amount.
	/// This value doesn't always equal to the `gas_burnt` multiplied by the gas price, because
	/// the prepaid gas price might be lower than the actual gas price and it creates a deficit.
	#[serde(with = "dec_format")]
	pub tokens_burnt: Balance,
	/// The id of the account on which the execution happens. For transaction this is signer_id,
	/// for receipt this is receiver_id.
	pub executor_id: AccountId,
	/// Execution status. Contains the result in case of successful execution.
	pub status: ExecutionStatusView,
	/// Execution metadata, versioned
	#[serde(default)]
	pub metadata: ExecutionMetadataView,
}

#[derive(
	BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone,
)]
pub enum ExecutionStatusView {
	/// The execution is pending or unknown.
	Unknown,
	/// The execution has failed.
	Failure(TxExecutionError),
	/// The final action succeeded and returned some value or an empty vec encoded in base64.
	SuccessValue(#[serde(with = "base64_format")] Vec<u8>),
	/// The final action of the receipt returned a promise or the signed transaction was converted
	/// to a receipt. Contains the receipt_id of the generated receipt.
	SuccessReceiptId(CryptoHash),
}

#[derive(
	BorshSerialize,
	BorshDeserialize,
	PartialEq,
	Clone,
	Eq,
	Debug,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct CostGasUsed {
	pub cost_category: String,
	pub cost: String,
	#[serde(with = "dec_format")]
	pub gas_used: Gas,
}

#[derive(
	BorshSerialize,
	BorshDeserialize,
	PartialEq,
	Clone,
	Eq,
	Debug,
	serde::Serialize,
	serde::Deserialize,
)]
pub struct ExecutionMetadataView {
	pub version: u32,
	pub gas_profile: Option<Vec<CostGasUsed>>,
}

// pub fn reconstruct_outcome_root() {
//     shard_outcome_root = compute_root(sha256(borsh(execution_outcome)), outcome_proof.proof)
// block_outcome_root = compute_root(sha256(borsh(shard_outcome_root)), outcome_root_proof)
// This outcome root must match the outcome root in block_header_lite.inner_lite.

// }
