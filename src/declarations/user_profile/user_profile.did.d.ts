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
/**
 * ============================================================================
 * User Profile Candid Interface
 * ============================================================================
 * This canister manages user profiles, quiz submissions, and staking state.
 * It operates as a "shard" that syncs with the central staking_hub.
 * Simplified version without interest/tier tracking.
 * User profile (simplified - no interest/tier fields)
 */
export interface UserProfile {
  'name' : string,
  'education' : string,
  /**
   * Personal Information
   */
  'email' : string,
  /**
   * Economy State
   */
  'staked_balance' : bigint,
  'gender' : string,
  'transaction_count' : bigint,
}
export interface UserProfileUpdate {
  'name' : string,
  'education' : string,
  'email' : string,
  'gender' : string,
}
export interface _SERVICE {
  /**
   * =========================================================================
   * Debug / Testing
   * =========================================================================
   */
  'debug_force_sync' : ActorMethod<[], { 'Ok' : null } | { 'Err' : string }>,
  'get_profile' : ActorMethod<[Principal], [] | [UserProfile]>,
  /**
   * =========================================================================
   * Shard Metrics
   * =========================================================================
   */
  'get_user_count' : ActorMethod<[], bigint>,
  /**
   * =========================================================================
   * Daily Stats
   * =========================================================================
   */
  'get_user_daily_status' : ActorMethod<[Principal], UserDailyStats>,
  /**
   * =========================================================================
   * Transaction History
   * =========================================================================
   */
  'get_user_transactions' : ActorMethod<[Principal], Array<TransactionRecord>>,
  'is_quiz_completed' : ActorMethod<[Principal, string], boolean>,
  /**
   * =========================================================================
   * User Registration & Profile
   * =========================================================================
   */
  'register_user' : ActorMethod<
    [UserProfileUpdate],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  /**
   * =========================================================================
   * Quiz & Learning
   * =========================================================================
   */
  'submit_quiz' : ActorMethod<
    [string, Uint8Array | number[]],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  /**
   * =========================================================================
   * Economy Functions
   * =========================================================================
   * Unstake returns 100% (no penalty)
   */
  'unstake' : ActorMethod<[bigint], { 'Ok' : bigint } | { 'Err' : string }>,
  'update_profile' : ActorMethod<
    [UserProfileUpdate],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
