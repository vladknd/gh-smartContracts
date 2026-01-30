import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ArchiveConfig {
  'trigger_threshold' : bigint,
  'is_configured' : boolean,
  'archive_canister_id' : [] | [Principal],
  'retention_limit' : bigint,
  'check_interval_secs' : bigint,
}
export interface InitArgs {
  'learning_content_id' : Principal,
  'staking_hub_id' : Principal,
}
export interface QuizCacheData {
  'question_count' : number,
  'content_id' : string,
  'version' : bigint,
  'answer_hashes' : Array<Uint8Array | number[]>,
}
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
export interface TransactionPage {
  'has_archive_data' : boolean,
  'source' : string,
  'total_pages' : number,
  'current_page' : number,
  'archive_canister_id' : [] | [Principal],
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
export interface UserListResult {
  'page_size' : number,
  'page' : number,
  'total_pages' : number,
  'users' : Array<UserSummary>,
  'total_count' : bigint,
  'has_more' : boolean,
}
export interface UserProfile {
  'name' : string,
  'education' : string,
  'email' : string,
  'staked_balance' : bigint,
  'archived_transaction_count' : bigint,
  'gender' : string,
  'verification_tier' : VerificationTier,
  'transaction_count' : bigint,
  'is_subscribed' : boolean,
}
export interface UserProfileUpdate {
  'name' : string,
  'education' : string,
  'email' : string,
  'gender' : string,
}
export interface UserSummary {
  'user_principal' : Principal,
  'name' : string,
  'email' : string,
  'staked_balance' : bigint,
  'verification_tier' : VerificationTier,
  'is_subscribed' : boolean,
}
export interface UserTimeStats {
  'weekly_earnings' : bigint,
  'monthly_earnings' : bigint,
  'daily_earnings' : bigint,
  'last_active_day' : bigint,
  'weekly_quizzes' : number,
  'yearly_quizzes' : number,
  'monthly_quizzes' : number,
  'daily_quizzes' : number,
  'yearly_earnings' : bigint,
}
export type VerificationTier = { 'KYC' : null } |
  { 'None' : null } |
  { 'Human' : null };
export interface _SERVICE {
  'admin_get_user_details' : ActorMethod<
    [Principal],
    { 'Ok' : [] | [UserProfile] } |
      { 'Err' : string }
  >,
  'admin_list_all_users' : ActorMethod<
    [number, number],
    { 'Ok' : UserListResult } |
      { 'Err' : string }
  >,
  'admin_set_kyc_status' : ActorMethod<
    [Principal, VerificationTier],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'admin_set_subscription' : ActorMethod<
    [Principal, boolean],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'admin_set_user_stats' : ActorMethod<
    [Principal, UserTimeStats],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'debug_force_sync' : ActorMethod<[], { 'Ok' : null } | { 'Err' : string }>,
  'debug_trigger_archive' : ActorMethod<
    [],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'get_archive_canister' : ActorMethod<[], Principal>,
  'get_archive_config' : ActorMethod<[], ArchiveConfig>,
  'get_kyc_manager_id' : ActorMethod<[], Principal>,
  'get_profile' : ActorMethod<[Principal], [] | [UserProfile]>,
  'get_subscription_manager_id' : ActorMethod<[], Principal>,
  'get_token_limits' : ActorMethod<[], TokenLimitsConfig>,
  'get_transactions_page' : ActorMethod<[Principal, number], TransactionPage>,
  'get_user_count' : ActorMethod<[], bigint>,
  'get_user_stats' : ActorMethod<[Principal], UserTimeStats>,
  'get_user_transactions' : ActorMethod<[Principal], Array<TransactionRecord>>,
  'internal_set_kyc_status' : ActorMethod<
    [Principal, VerificationTier],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'internal_set_subscription' : ActorMethod<
    [Principal, boolean],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'internal_sync_kyc_manager' : ActorMethod<[Principal], undefined>,
  'internal_sync_subscription_manager' : ActorMethod<[Principal], undefined>,
  'is_quiz_completed' : ActorMethod<[Principal, string], boolean>,
  'is_user_registered' : ActorMethod<[Principal], boolean>,
  'receive_full_quiz_cache' : ActorMethod<
    [Array<[string, QuizCacheData]>],
    undefined
  >,
  'receive_quiz_cache' : ActorMethod<[string, QuizCacheData], undefined>,
  'receive_token_limits' : ActorMethod<[TokenLimitsConfig], undefined>,
  'register_user' : ActorMethod<
    [UserProfileUpdate],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_archive_canister' : ActorMethod<
    [Principal],
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
  'whoami' : ActorMethod<[], Principal>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
