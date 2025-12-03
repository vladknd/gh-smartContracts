import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'learning_content_id' : Principal,
  'staking_hub_id' : Principal,
}
export interface UserDailyStats {
  'quizzes_taken' : number,
  'tokens_earned' : bigint,
  'day_index' : bigint,
}
export interface UserProfile {
  'name' : string,
  'education' : string,
  'email' : string,
  'staked_balance' : bigint,
  'last_reward_index' : bigint,
  'gender' : string,
}
export interface _SERVICE {
  'debug_force_sync' : ActorMethod<[], { 'Ok' : null } | { 'Err' : string }>,
  'get_profile' : ActorMethod<[Principal], [] | [UserProfile]>,
  'get_user_daily_status' : ActorMethod<[Principal], UserDailyStats>,
  'is_quiz_completed' : ActorMethod<[Principal, string], boolean>,
  'submit_quiz' : ActorMethod<
    [string, Uint8Array | number[]],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'unstake' : ActorMethod<[bigint], { 'Ok' : bigint } | { 'Err' : string }>,
  'update_profile' : ActorMethod<[UserProfile], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
