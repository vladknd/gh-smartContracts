import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'learning_content_id' : Principal,
  'staking_hub_id' : Principal,
}
export interface TransactionRecord {
  'timestamp' : bigint,
  'tx_type' : TransactionType,
  'amount' : bigint,
}
export type TransactionType = { 'Unstake' : null } |
  { 'QuizReward' : null };
export interface UserDailyStats {
  'quizzes_taken' : number,
  'tokens_earned' : bigint,
  'day_index' : bigint,
}
export interface UserProfile {
  'initial_stake_time' : bigint,
  'name' : string,
  'education' : string,
  'unclaimed_interest' : bigint,
  'email' : string,
  'staked_balance' : bigint,
  'last_reward_index' : bigint,
  'tier_start_index' : bigint,
  'gender' : string,
  'current_tier' : number,
  'transaction_count' : bigint,
}
export interface UserProfileUpdate {
  'name' : string,
  'education' : string,
  'email' : string,
  'gender' : string,
}
export interface _SERVICE {
  'claim_rewards' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'debug_force_sync' : ActorMethod<[], { 'Ok' : null } | { 'Err' : string }>,
  'get_profile' : ActorMethod<[Principal], [] | [UserProfile]>,
  'get_user_count' : ActorMethod<[], bigint>,
  'get_user_daily_status' : ActorMethod<[Principal], UserDailyStats>,
  'get_user_transactions' : ActorMethod<[Principal], Array<TransactionRecord>>,
  'is_quiz_completed' : ActorMethod<[Principal, string], boolean>,
  'register_user' : ActorMethod<
    [UserProfileUpdate],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'submit_quiz' : ActorMethod<
    [string, Uint8Array | number[]],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'unstake' : ActorMethod<[bigint], { 'Ok' : bigint } | { 'Err' : string }>,
  'update_profile' : ActorMethod<
    [UserProfileUpdate],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
