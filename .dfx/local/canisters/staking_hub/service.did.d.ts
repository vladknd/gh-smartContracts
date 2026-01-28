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
  'archive_canister_wasm' : [] | [Uint8Array | number[]],
  'ledger_id' : Principal,
  'user_profile_wasm' : Uint8Array | number[],
}
export interface QuizCacheData {
  'question_count' : number,
  'content_id' : string,
  'version' : bigint,
  'answer_hashes' : Array<Uint8Array | number[]>,
}
export interface ShardInfo {
  'user_count' : bigint,
  'status' : ShardStatus,
  'canister_id' : Principal,
  'created_at' : bigint,
  'archive_canister_id' : [] | [Principal],
}
export type ShardStatus = { 'Full' : null } |
  { 'Active' : null };
export interface TokenLimits {
  'max_monthly_tokens' : bigint,
  'max_yearly_tokens' : bigint,
  'max_daily_tokens' : bigint,
  'max_weekly_tokens' : bigint,
}
export interface TokenLimitsConfig {
  'reward_amount' : bigint,
  'pass_threshold_percent' : number,
  'max_daily_attempts' : number,
  'regular_limits' : TokenLimits,
  'version' : bigint,
  'subscribed_limits' : TokenLimits,
}
export interface _SERVICE {
  'add_allowed_minter' : ActorMethod<[Principal], undefined>,
  'admin_broadcast_kyc_manager' : ActorMethod<
    [Principal],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'admin_broadcast_subscription_manager' : ActorMethod<
    [Principal],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'admin_set_user_shard' : ActorMethod<
    [Principal, Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'are_board_shares_locked' : ActorMethod<[], boolean>,
  'distribute_quiz_cache' : ActorMethod<
    [string, QuizCacheData],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'distribute_token_limits' : ActorMethod<
    [TokenLimitsConfig],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'ensure_capacity' : ActorMethod<
    [],
    { 'Ok' : [] | [Principal] } |
      { 'Err' : string }
  >,
  'fetch_user_voting_power' : ActorMethod<[Principal], bigint>,
  'get_active_shards' : ActorMethod<[], Array<ShardInfo>>,
  'get_archive_for_shard' : ActorMethod<[Principal], [] | [Principal]>,
  'get_board_member_count' : ActorMethod<[], bigint>,
  'get_board_member_share' : ActorMethod<[Principal], [] | [number]>,
  'get_board_member_shares' : ActorMethod<[], Array<BoardMemberShare>>,
  'get_config' : ActorMethod<[], [Principal, Principal, boolean]>,
  'get_global_stats' : ActorMethod<[], GlobalStats>,
  'get_kyc_manager_id' : ActorMethod<[], Principal>,
  'get_limits' : ActorMethod<[], [bigint, bigint]>,
  'get_shard_count' : ActorMethod<[], bigint>,
  'get_shard_for_new_user' : ActorMethod<[], [] | [Principal]>,
  'get_shards' : ActorMethod<[], Array<ShardInfo>>,
  'get_subscription_manager_id' : ActorMethod<[], Principal>,
  'get_token_limits' : ActorMethod<[], TokenLimitsConfig>,
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
  'register_shard' : ActorMethod<[Principal, [] | [Principal]], undefined>,
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
  'update_token_limits' : ActorMethod<
    [
      [] | [bigint],
      [] | [number],
      [] | [number],
      [] | [TokenLimits],
      [] | [TokenLimits],
    ],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
