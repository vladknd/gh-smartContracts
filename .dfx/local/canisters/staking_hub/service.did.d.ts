import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface BoardMemberShare {
  'member' : Principal,
  'percentage' : number,
}
export interface GlobalStats {
  'total_staked' : bigint,
  'total_allocated' : bigint,
  'total_unstaked' : bigint,
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
export interface _SERVICE {
  'add_allowed_minter' : ActorMethod<[Principal], undefined>,
  'admin_set_user_shard' : ActorMethod<
    [Principal, Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'are_board_shares_locked' : ActorMethod<[], boolean>,
  'ensure_capacity' : ActorMethod<
    [],
    { 'Ok' : [] | [Principal] } |
      { 'Err' : string }
  >,
  'fetch_voting_power' : ActorMethod<[Principal], bigint>,
  'get_active_shards' : ActorMethod<[], Array<ShardInfo>>,
  'get_board_member_count' : ActorMethod<[], bigint>,
  'get_board_member_share' : ActorMethod<[Principal], [] | [number]>,
  'get_board_member_shares' : ActorMethod<[], Array<BoardMemberShare>>,
  'get_config' : ActorMethod<[], [Principal, Principal, boolean]>,
  'get_global_stats' : ActorMethod<[], GlobalStats>,
  'get_limits' : ActorMethod<[], [bigint, bigint]>,
  'get_shard_count' : ActorMethod<[], bigint>,
  'get_shard_for_new_user' : ActorMethod<[], [] | [Principal]>,
  'get_shards' : ActorMethod<[], Array<ShardInfo>>,
  'get_tokenomics' : ActorMethod<[], [bigint, bigint, bigint, bigint]>,
  'get_total_voting_power' : ActorMethod<[], bigint>,
  'get_user_shard' : ActorMethod<[Principal], [] | [Principal]>,
  'get_vuc' : ActorMethod<[], bigint>,
  'is_board_member' : ActorMethod<[Principal], boolean>,
  'is_registered_shard' : ActorMethod<[Principal], boolean>,
  'lock_board_member_shares' : ActorMethod<
    [],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'process_unstake' : ActorMethod<
    [Principal, bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'register_user_location' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_board_member_shares' : ActorMethod<
    [Array<BoardMemberShare>],
    { 'Ok' : null } |
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
