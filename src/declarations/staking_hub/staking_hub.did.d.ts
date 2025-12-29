import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

/**
 * Global statistics (simplified - no interest/tier tracking)
 */
export interface GlobalStats {
  'total_staked' : bigint,
  'total_allocated' : bigint,
  'total_unstaked' : bigint,
}
/**
 * ============================================================================
 * Staking Hub Candid Interface
 * ============================================================================
 * This canister manages global staking statistics and coordinates
 * user_profile shard canisters. Simplified version without interest/penalties.
 * Arguments for canister initialization
 */
export interface InitArgs {
  'learning_content_id' : Principal,
  'ledger_id' : Principal,
  'user_profile_wasm' : Uint8Array | number[],
}
/**
 * Information about a user_profile shard canister
 */
export interface ShardInfo {
  'user_count' : bigint,
  'status' : ShardStatus,
  'canister_id' : Principal,
  'created_at' : bigint,
}
/**
 * Shard operational status
 */
export type ShardStatus = { 'Full' : null } |
  { 'Active' : null };
export interface _SERVICE {
  'add_allowed_minter' : ActorMethod<[Principal], undefined>,
  'ensure_capacity' : ActorMethod<
    [],
    { 'Ok' : [] | [Principal] } |
      { 'Err' : string }
  >,
  'get_active_shards' : ActorMethod<[], Array<ShardInfo>>,
  'get_config' : ActorMethod<[], [Principal, Principal, boolean]>,
  'get_global_stats' : ActorMethod<[], GlobalStats>,
  'get_limits' : ActorMethod<[], [bigint, bigint]>,
  'get_shard_count' : ActorMethod<[], bigint>,
  'get_shard_for_new_user' : ActorMethod<[], [] | [Principal]>,
  'get_shards' : ActorMethod<[], Array<ShardInfo>>,
  'is_registered_shard' : ActorMethod<[Principal], boolean>,
  'process_unstake' : ActorMethod<
    [Principal, bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'sync_shard' : ActorMethod<
    [bigint, bigint, bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'update_shard_user_count' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
