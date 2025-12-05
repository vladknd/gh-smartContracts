import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface GlobalStats {
  'cumulative_reward_index' : bigint,
  'total_staked' : bigint,
  'total_allocated' : bigint,
  'total_unstaked' : bigint,
  'interest_pool' : bigint,
  'total_rewards_distributed' : bigint,
}
export interface InitArgs { 'ledger_id' : Principal }
export interface _SERVICE {
  'add_allowed_minter' : ActorMethod<[Principal], undefined>,
  'distribute_interest' : ActorMethod<
    [],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'get_global_stats' : ActorMethod<[], GlobalStats>,
  'process_unstake' : ActorMethod<
    [Principal, bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'remove_allowed_minter' : ActorMethod<[Principal], undefined>,
  'sync_shard' : ActorMethod<
    [bigint, bigint, bigint, bigint],
    { 'Ok' : [bigint, bigint] } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
