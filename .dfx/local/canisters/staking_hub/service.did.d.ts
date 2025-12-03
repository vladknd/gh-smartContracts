import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface GlobalStats {
  'cumulative_reward_index' : bigint,
  'total_staked' : bigint,
  'total_unstaked' : bigint,
  'interest_pool' : bigint,
}
export interface InitArgs { 'ledger_id' : Principal }
export interface _SERVICE {
  'distribute_interest' : ActorMethod<
    [],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'get_global_stats' : ActorMethod<[], GlobalStats>,
  'get_user_stats' : ActorMethod<[Principal], [bigint, bigint]>,
  'get_voting_power' : ActorMethod<[Principal], bigint>,
  'stake_rewards' : ActorMethod<[Principal, bigint], undefined>,
  'unstake' : ActorMethod<[bigint], { 'Ok' : bigint } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
