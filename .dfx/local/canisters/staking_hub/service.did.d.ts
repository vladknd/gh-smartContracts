import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface GlobalStats {
  'cumulative_reward_index' : bigint,
  'tier_reward_indexes' : Array<bigint>,
  'total_staked' : bigint,
  'total_allocated' : bigint,
  'tier_staked' : BigUint64Array | bigint[],
  'total_unstaked' : bigint,
  'interest_pool' : bigint,
  'total_rewards_distributed' : bigint,
}
export interface InitArgs {
  'learning_content_id' : Principal,
  'ledger_id' : Principal,
  'user_profile_wasm' : Uint8Array | number[],
}
export interface ShardInfo {
  'user_count' : bigint,
  'status' : ShardStatus,
  'canister_id' : Principal,
  'created_at' : bigint,
}
export type ShardStatus = { 'Full' : null } |
  { 'Active' : null };
export type TierDeltas = BigInt64Array | bigint[];
export type TierIndexes = Array<bigint>;
export interface _SERVICE {
  'add_allowed_minter' : ActorMethod<[Principal], undefined>,
  'distribute_interest' : ActorMethod<
    [],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
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
    [TierDeltas, bigint, bigint, bigint],
    { 'Ok' : [bigint, TierIndexes] } |
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
