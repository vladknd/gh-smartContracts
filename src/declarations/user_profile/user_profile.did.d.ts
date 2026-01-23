import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

/**
 * Archive configuration info
 */
export interface ArchiveConfig {
  /**
   * Transactions kept locally per user
   */
  'trigger_threshold' : bigint,
  'is_configured' : boolean,
  /**
   * Periodic archive check interval
   */
  'archive_canister_id' : Principal,
  'retention_limit' : bigint,
  /**
   * Threshold for immediate archive
   */
  'check_interval_secs' : bigint,
}
/**
 * Cached quiz configuration (stored locally, updated via staking_hub)
 */
export interface CachedQuizConfig {
  'max_daily_quizzes' : number,
  'reward_amount' : bigint,
  'max_monthly_quizzes' : number,
  'pass_threshold_percent' : number,
  'max_daily_attempts' : number,
  'version' : bigint,
  'max_weekly_quizzes' : number,
  'max_yearly_quizzes' : number,
}
export interface InitArgs {
  'learning_content_id' : Principal,
  'staking_hub_id' : Principal,
}
/**
 * Paginated transaction response with archive info
 */
export interface TransactionPage {
  'source' : string,
  'archive_canister_id' : Principal,
  'transactions' : Array<TransactionRecord>,
  'archived_count' : bigint,
  'local_count' : bigint,
  'total_count' : bigint,
}
export interface TransactionRecord {
  'timestamp' : bigint,
  'tx_type' : TransactionType,
  'amount' : bigint,
}
export type TransactionType = { 'Unstake' : null } |
  { 'QuizReward' : null };
/**
 * Result of admin user listing with pagination info
 */
export interface UserListResult {
  'page_size' : number,
  'page' : number,
  'users' : Array<UserSummary>,
  'total_count' : bigint,
  'has_more' : boolean,
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
/**
 * Summary of a registered user for admin listing
 */
export interface UserSummary {
  'user_principal' : Principal,
  'name' : string,
  'email' : string,
  'staked_balance' : bigint,
  'verification_tier' : VerificationTier,
}
export interface UserTimeStats {
  'weekly_earnings' : bigint,
  'monthly_earnings' : bigint,
  'daily_earnings' : bigint,
  'last_active_day' : bigint,
  /**
   * Weekly
   */
  'weekly_quizzes' : number,
  /**
   * Yearly
   */
  'yearly_quizzes' : number,
  /**
   * Monthly
   */
  'monthly_quizzes' : number,
  /**
   * Daily
   */
  'daily_quizzes' : number,
  'yearly_earnings' : bigint,
}
/**
 * Verification tier for users
 */
export type VerificationTier = {
    /**
     * DecideID verified
     */
    'KYC' : null
  } |
  { 'None' : null } |
  {
    /**
     * Fresh user
     */
    'Human' : null
  };
export interface _SERVICE {
  'admin_get_user_details' : ActorMethod<
    [Principal],
    { 'Ok' : [] | [UserProfile] } |
      { 'Err' : string }
  >,
  /**
   * =========================================================================
   * Admin Debug Endpoints (Controller-Only)
   * =========================================================================
   * For debugging authentication issues
   */
  'admin_list_all_users' : ActorMethod<
    [number, number],
    { 'Ok' : UserListResult } |
      { 'Err' : string }
  >,
  /**
   * =========================================================================
   * Debug / Testing
   * =========================================================================
   */
  'debug_force_sync' : ActorMethod<[], { 'Ok' : null } | { 'Err' : string }>,
  'get_archive_canister' : ActorMethod<[], Principal>,
  'get_archive_config' : ActorMethod<[], ArchiveConfig>,
  'get_cached_quiz_config' : ActorMethod<[], CachedQuizConfig>,
  'get_profile' : ActorMethod<[Principal], [] | [UserProfile]>,
  'get_transactions_page' : ActorMethod<[Principal, number], TransactionPage>,
  /**
   * =========================================================================
   * Shard Metrics
   * =========================================================================
   */
  'get_user_count' : ActorMethod<[], bigint>,
  'get_user_daily_status' : ActorMethod<[Principal], UserTimeStats>,
  /**
   * =========================================================================
   * User Stats
   * =========================================================================
   */
  'get_user_stats' : ActorMethod<[Principal], UserTimeStats>,
  /**
   * =========================================================================
   * Transaction History
   * =========================================================================
   */
  'get_user_transactions' : ActorMethod<[Principal], Array<TransactionRecord>>,
  'is_quiz_completed' : ActorMethod<[Principal, string], boolean>,
  'is_user_registered' : ActorMethod<[Principal], boolean>,
  /**
   * Quiz config caching (updated by staking_hub when governance proposals pass)
   */
  'receive_quiz_config' : ActorMethod<[CachedQuizConfig], undefined>,
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
   * Archive Operations
   * =========================================================================
   */
  'set_archive_canister' : ActorMethod<
    [Principal],
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
  'trigger_archive' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
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
  /**
   * Public debug helpers
   */
  'whoami' : ActorMethod<[], Principal>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
