import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Account {
  'owner' : Principal,
  'subaccount' : [] | [Subaccount],
}
export interface Allowance {
  'allowance' : bigint,
  'expires_at' : [] | [bigint],
}
export interface AllowanceArgs { 'account' : Account, 'spender' : Account }
export interface Approve {
  'fee' : [] | [bigint],
  'from' : Account,
  'memo' : [] | [Uint8Array | number[]],
  'created_at_time' : [] | [bigint],
  'amount' : bigint,
  'expected_allowance' : [] | [bigint],
  'expires_at' : [] | [bigint],
  'spender' : Account,
}
export interface ApproveArgs {
  'fee' : [] | [bigint],
  'memo' : [] | [Uint8Array | number[]],
  'from_subaccount' : [] | [Uint8Array | number[]],
  'created_at_time' : [] | [bigint],
  'amount' : bigint,
  'expected_allowance' : [] | [bigint],
  'expires_at' : [] | [bigint],
  'spender' : Account,
}
export type ApproveError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'Duplicate' : { 'duplicate_of' : bigint } } |
  { 'BadFee' : { 'expected_fee' : bigint } } |
  { 'AllowanceChanged' : { 'current_allowance' : bigint } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'Expired' : { 'ledger_time' : bigint } } |
  { 'InsufficientFunds' : { 'balance' : bigint } };
export type ApproveResult = { 'Ok' : bigint } |
  { 'Err' : ApproveError };
export type Block = Value;
export type BlockIndex = bigint;
/**
 * A prefix of the block range specified in the [GetBlocksArgs] request.
 */
export interface BlockRange {
  /**
   * A prefix of the requested block range.
   * The index of the first block is equal to [GetBlocksArgs.start].
   * 
   * Note that the number of blocks might be less than the requested
   * [GetBlocksArgs.length] for various reasons, for example:
   * 
   * 1. The query might have hit the replica with an outdated state
   * that doesn't have the whole range yet.
   * 2. The requested range is too large to fit into a single reply.
   * 
   * NOTE: the list of blocks can be empty if:
   * 
   * 1. [GetBlocksArgs.length] was zero.
   * 2. [GetBlocksArgs.start] was larger than the last block known to
   * the canister.
   */
  'blocks' : Array<Block>,
}
export interface Burn {
  'from' : Account,
  'memo' : [] | [Uint8Array | number[]],
  'created_at_time' : [] | [bigint],
  'amount' : bigint,
  'spender' : [] | [Account],
}
export type ChangeFeeCollector = { 'SetTo' : Account } |
  { 'Unset' : null };
/**
 * Certificate for the block at `block_index`.
 */
export interface DataCertificate {
  'certificate' : [] | [Uint8Array | number[]],
  'hash_tree' : Uint8Array | number[],
}
/**
 * Number of nanoseconds between two [Timestamp]s.
 */
export type Duration = bigint;
export interface FeatureFlags { 'icrc2' : boolean }
export interface GetBlocksArgs {
  /**
   * The index of the first block to fetch.
   */
  'start' : BlockIndex,
  /**
   * Max number of blocks to fetch.
   */
  'length' : bigint,
}
/**
 * The result of a "get_blocks" call.
 */
export interface GetBlocksResponse {
  /**
   * System certificate for the hash of the latest block in the chain.
   * Only present if `get_blocks` is called in a non-replicated query context.
   */
  'certificate' : [] | [Uint8Array | number[]],
  /**
   * The index of the first block in "blocks".
   * If the blocks vector is empty, the exact value of this field is not specified.
   */
  'first_index' : BlockIndex,
  /**
   * List of blocks that were available in the ledger when it processed the call.
   * 
   * The blocks form a contiguous range, with the first block having index
   * [first_block_index] (see below), and the last block having index
   * [first_block_index] + len(blocks) - 1.
   * 
   * The block range can be an arbitrary sub-range of the originally requested range.
   */
  'blocks' : Array<Block>,
  /**
   * The total number of blocks in the chain.
   * If the chain length is positive, the index of the last block is `chain_len - 1`.
   */
  'chain_length' : bigint,
  /**
   * Encoding of instructions for fetching archived blocks.
   */
  'archived_blocks' : Array<
    {
      /**
       * Callback to fetch the archived blocks.
       */
      'callback' : [Principal, string],
      /**
       * The index of the first archived block.
       */
      'start' : BlockIndex,
      /**
       * The number of blocks that can be fetched.
       */
      'length' : bigint,
    }
  >,
}
export interface GetTransactionsRequest {
  /**
   * The index of the first tx to fetch.
   */
  'start' : TxIndex,
  /**
   * The number of transactions to fetch.
   */
  'length' : bigint,
}
export interface GetTransactionsResponse {
  /**
   * The index of the first transaction in [transactions].
   * If the transaction vector is empty, the exact value of this field is not specified.
   */
  'first_index' : TxIndex,
  /**
   * The total number of transactions in the log.
   */
  'log_length' : bigint,
  /**
   * List of transaction that were available in the ledger when it processed the call.
   * 
   * The transactions form a contiguous range, with the first transaction having index
   * [first_index] (see below), and the last transaction having index
   * [first_index] + len(transactions) - 1.
   * 
   * The transaction range can be an arbitrary sub-range of the originally requested range.
   */
  'transactions' : Array<Transaction>,
  /**
   * Encoding of instructions for fetching archived transactions whose indices fall into the
   * requested range.
   * 
   * For each entry `e` in [archived_transactions], `[e.from, e.from + len)` is a sub-range
   * of the originally requested transaction range.
   */
  'archived_transactions' : Array<
    {
      /**
       * The function you should call to fetch the archived transactions.
       * The range of the transaction accessible using this function is given by [from]
       * and [len] fields above.
       */
      'callback' : [Principal, string],
      /**
       * The index of the first archived transaction you can fetch using the [callback].
       */
      'start' : TxIndex,
      /**
       * The number of transactions you can fetch using the callback.
       */
      'length' : bigint,
    }
  >,
}
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
  'status_code' : number,
}
/**
 * The initialization parameters of the Ledger
 */
export interface InitArgs {
  'decimals' : [] | [number],
  'token_symbol' : string,
  'transfer_fee' : bigint,
  'metadata' : Array<[string, MetadataValue]>,
  'minting_account' : Account,
  'initial_balances' : Array<[Account, bigint]>,
  'maximum_number_of_accounts' : [] | [bigint],
  'accounts_overflow_trim_quantity' : [] | [bigint],
  'fee_collector_account' : [] | [Account],
  'archive_options' : {
    'num_blocks_to_archive' : bigint,
    'max_transactions_per_response' : [] | [bigint],
    'trigger_threshold' : bigint,
    'max_message_size_bytes' : [] | [bigint],
    'cycles_for_archive_creation' : [] | [bigint],
    'node_max_memory_size_bytes' : [] | [bigint],
    'controller_id' : Principal,
  },
  'max_memo_length' : [] | [number],
  'token_name' : string,
  'feature_flags' : [] | [FeatureFlags],
}
export type LedgerArg = { 'Upgrade' : [] | [UpgradeArgs] } |
  { 'Init' : InitArgs };
export type Map = Array<[string, Value]>;
/**
 * The value returned from the [icrc1_metadata] endpoint.
 */
export type MetadataValue = { 'Int' : bigint } |
  { 'Nat' : bigint } |
  { 'Blob' : Uint8Array | number[] } |
  { 'Text' : string };
export interface Mint {
  'to' : Account,
  'memo' : [] | [Uint8Array | number[]],
  'created_at_time' : [] | [bigint],
  'amount' : bigint,
}
/**
 * A function for fetching archived transaction.
 */
export type QueryArchiveFn = ActorMethod<
  [GetTransactionsRequest],
  TransactionRange
>;
/**
 * A function for fetching archived blocks.
 */
export type QueryBlockArchiveFn = ActorMethod<[GetBlocksArgs], BlockRange>;
export interface StandardRecord { 'url' : string, 'name' : string }
export type Subaccount = Uint8Array | number[];
/**
 * Number of nanoseconds since the UNIX epoch in UTC timezone.
 */
export type Timestamp = bigint;
export type Tokens = bigint;
export interface Transaction {
  'burn' : [] | [Burn],
  'kind' : string,
  'mint' : [] | [Mint],
  'approve' : [] | [Approve],
  'timestamp' : bigint,
  'transfer' : [] | [Transfer],
}
/**
 * A prefix of the transaction range specified in the [GetTransactionsRequest] request.
 */
export interface TransactionRange {
  /**
   * A prefix of the requested transaction range.
   * The index of the first transaction is equal to [GetTransactionsRequest.from].
   * 
   * Note that the number of transactions might be less than the requested
   * [GetTransactionsRequest.length] for various reasons, for example:
   * 
   * 1. The query might have hit the replica with an outdated state
   * that doesn't have the whole range yet.
   * 2. The requested range is too large to fit into a single reply.
   * 
   * NOTE: the list of transactions can be empty if:
   * 
   * 1. [GetTransactionsRequest.length] was zero.
   * 2. [GetTransactionsRequest.from] was larger than the last transaction known to
   * the canister.
   */
  'transactions' : Array<Transaction>,
}
export interface Transfer {
  'to' : Account,
  'fee' : [] | [bigint],
  'from' : Account,
  'memo' : [] | [Uint8Array | number[]],
  'created_at_time' : [] | [bigint],
  'amount' : bigint,
  'spender' : [] | [Account],
}
export interface TransferArg {
  'to' : Account,
  'fee' : [] | [Tokens],
  'memo' : [] | [Uint8Array | number[]],
  'from_subaccount' : [] | [Subaccount],
  'created_at_time' : [] | [Timestamp],
  'amount' : Tokens,
}
export type TransferError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'BadBurn' : { 'min_burn_amount' : Tokens } } |
  { 'Duplicate' : { 'duplicate_of' : BlockIndex } } |
  { 'BadFee' : { 'expected_fee' : Tokens } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'InsufficientFunds' : { 'balance' : Tokens } };
export interface TransferFromArgs {
  'to' : Account,
  'fee' : [] | [Tokens],
  'spender_subaccount' : [] | [Subaccount],
  'from' : Account,
  'memo' : [] | [Uint8Array | number[]],
  'created_at_time' : [] | [Timestamp],
  'amount' : Tokens,
}
export type TransferFromError = {
    'GenericError' : { 'message' : string, 'error_code' : bigint }
  } |
  { 'TemporarilyUnavailable' : null } |
  { 'InsufficientAllowance' : { 'allowance' : Tokens } } |
  { 'BadBurn' : { 'min_burn_amount' : Tokens } } |
  { 'Duplicate' : { 'duplicate_of' : BlockIndex } } |
  { 'BadFee' : { 'expected_fee' : Tokens } } |
  { 'CreatedInFuture' : { 'ledger_time' : bigint } } |
  { 'TooOld' : null } |
  { 'InsufficientFunds' : { 'balance' : Tokens } };
export type TransferFromResult = { 'Ok' : BlockIndex } |
  { 'Err' : TransferFromError };
export type TransferResult = { 'Ok' : BlockIndex } |
  { 'Err' : TransferError };
export type TxIndex = bigint;
export interface UpgradeArgs {
  'token_symbol' : [] | [string],
  'transfer_fee' : [] | [bigint],
  'metadata' : [] | [Array<[string, MetadataValue]>],
  'maximum_number_of_accounts' : [] | [bigint],
  'accounts_overflow_trim_quantity' : [] | [bigint],
  'change_fee_collector' : [] | [ChangeFeeCollector],
  'max_memo_length' : [] | [number],
  'token_name' : [] | [string],
  'feature_flags' : [] | [FeatureFlags],
}
export type Value = { 'Int' : bigint } |
  { 'Map' : Map } |
  { 'Nat' : bigint } |
  { 'Nat64' : bigint } |
  { 'Blob' : Uint8Array | number[] } |
  { 'Text' : string } |
  { 'Array' : Array<Value> };
export interface _SERVICE {
  'get_blocks' : ActorMethod<[GetBlocksArgs], GetBlocksResponse>,
  'get_data_certificate' : ActorMethod<[], DataCertificate>,
  'get_transactions' : ActorMethod<
    [GetTransactionsRequest],
    GetTransactionsResponse
  >,
  'icrc1_balance_of' : ActorMethod<[Account], Tokens>,
  'icrc1_decimals' : ActorMethod<[], number>,
  'icrc1_fee' : ActorMethod<[], Tokens>,
  'icrc1_metadata' : ActorMethod<[], Array<[string, MetadataValue]>>,
  'icrc1_minting_account' : ActorMethod<[], [] | [Account]>,
  'icrc1_name' : ActorMethod<[], string>,
  'icrc1_supported_standards' : ActorMethod<[], Array<StandardRecord>>,
  'icrc1_symbol' : ActorMethod<[], string>,
  'icrc1_total_supply' : ActorMethod<[], Tokens>,
  'icrc1_transfer' : ActorMethod<[TransferArg], TransferResult>,
  'icrc2_allowance' : ActorMethod<[AllowanceArgs], Allowance>,
  'icrc2_approve' : ActorMethod<[ApproveArgs], ApproveResult>,
  'icrc2_transfer_from' : ActorMethod<[TransferFromArgs], TransferFromResult>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
