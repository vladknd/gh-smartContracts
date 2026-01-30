export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  const VerificationTier = IDL.Variant({
    'KYC' : IDL.Null,
    'None' : IDL.Null,
    'Human' : IDL.Null,
  });
  const UserProfile = IDL.Record({
    'name' : IDL.Text,
    'education' : IDL.Text,
    'email' : IDL.Text,
    'staked_balance' : IDL.Nat64,
    'archived_transaction_count' : IDL.Nat64,
    'gender' : IDL.Text,
    'verification_tier' : VerificationTier,
    'transaction_count' : IDL.Nat64,
    'is_subscribed' : IDL.Bool,
  });
  const UserSummary = IDL.Record({
    'user_principal' : IDL.Principal,
    'name' : IDL.Text,
    'email' : IDL.Text,
    'staked_balance' : IDL.Nat64,
    'verification_tier' : VerificationTier,
    'is_subscribed' : IDL.Bool,
  });
  const UserListResult = IDL.Record({
    'page_size' : IDL.Nat32,
    'page' : IDL.Nat32,
    'total_pages' : IDL.Nat32,
    'users' : IDL.Vec(UserSummary),
    'total_count' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const UserTimeStats = IDL.Record({
    'weekly_earnings' : IDL.Nat64,
    'monthly_earnings' : IDL.Nat64,
    'daily_earnings' : IDL.Nat64,
    'last_active_day' : IDL.Nat64,
    'weekly_quizzes' : IDL.Nat8,
    'yearly_quizzes' : IDL.Nat16,
    'monthly_quizzes' : IDL.Nat8,
    'daily_quizzes' : IDL.Nat8,
    'yearly_earnings' : IDL.Nat64,
  });
  const ArchiveConfig = IDL.Record({
    'trigger_threshold' : IDL.Nat64,
    'is_configured' : IDL.Bool,
    'archive_canister_id' : IDL.Opt(IDL.Principal),
    'retention_limit' : IDL.Nat64,
    'check_interval_secs' : IDL.Nat64,
  });
  const TokenLimits = IDL.Record({
    'max_monthly_tokens' : IDL.Nat64,
    'max_yearly_tokens' : IDL.Nat64,
    'max_daily_tokens' : IDL.Nat64,
    'max_weekly_tokens' : IDL.Nat64,
  });
  const TokenLimitsConfig = IDL.Record({
    'reward_amount' : IDL.Nat64,
    'pass_threshold_percent' : IDL.Nat8,
    'max_daily_attempts' : IDL.Nat8,
    'regular_limits' : TokenLimits,
    'version' : IDL.Nat64,
    'subscribed_limits' : TokenLimits,
  });
  const TransactionType = IDL.Variant({
    'Unstake' : IDL.Null,
    'QuizReward' : IDL.Null,
  });
  const TransactionRecord = IDL.Record({
    'timestamp' : IDL.Nat64,
    'tx_type' : TransactionType,
    'amount' : IDL.Nat64,
  });
  const TransactionPage = IDL.Record({
    'has_archive_data' : IDL.Bool,
    'source' : IDL.Text,
    'total_pages' : IDL.Nat32,
    'current_page' : IDL.Nat32,
    'archive_canister_id' : IDL.Opt(IDL.Principal),
    'transactions' : IDL.Vec(TransactionRecord),
    'archived_count' : IDL.Nat64,
    'local_count' : IDL.Nat64,
    'total_count' : IDL.Nat64,
  });
  const QuizCacheData = IDL.Record({
    'question_count' : IDL.Nat8,
    'content_id' : IDL.Text,
    'version' : IDL.Nat64,
    'answer_hashes' : IDL.Vec(IDL.Vec(IDL.Nat8)),
  });
  const UserProfileUpdate = IDL.Record({
    'name' : IDL.Text,
    'education' : IDL.Text,
    'email' : IDL.Text,
    'gender' : IDL.Text,
  });
  return IDL.Service({
    'admin_get_user_details' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Opt(UserProfile), 'Err' : IDL.Text })],
        ['query'],
      ),
    'admin_list_all_users' : IDL.Func(
        [IDL.Nat32, IDL.Nat32],
        [IDL.Variant({ 'Ok' : UserListResult, 'Err' : IDL.Text })],
        ['query'],
      ),
    'admin_set_kyc_status' : IDL.Func(
        [IDL.Principal, VerificationTier],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'admin_set_subscription' : IDL.Func(
        [IDL.Principal, IDL.Bool],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'admin_set_user_stats' : IDL.Func(
        [IDL.Principal, UserTimeStats],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'debug_force_sync' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'debug_trigger_archive' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'get_archive_canister' : IDL.Func([], [IDL.Principal], ['query']),
    'get_archive_config' : IDL.Func([], [ArchiveConfig], ['query']),
    'get_kyc_manager_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_profile' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(UserProfile)],
        ['query'],
      ),
    'get_subscription_manager_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_token_limits' : IDL.Func([], [TokenLimitsConfig], ['query']),
    'get_transactions_page' : IDL.Func(
        [IDL.Principal, IDL.Nat32],
        [TransactionPage],
        ['query'],
      ),
    'get_user_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_user_stats' : IDL.Func([IDL.Principal], [UserTimeStats], ['query']),
    'get_user_transactions' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(TransactionRecord)],
        ['query'],
      ),
    'internal_set_kyc_status' : IDL.Func(
        [IDL.Principal, VerificationTier],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'internal_set_subscription' : IDL.Func(
        [IDL.Principal, IDL.Bool],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'internal_sync_kyc_manager' : IDL.Func([IDL.Principal], [], []),
    'internal_sync_subscription_manager' : IDL.Func([IDL.Principal], [], []),
    'is_quiz_completed' : IDL.Func(
        [IDL.Principal, IDL.Text],
        [IDL.Bool],
        ['query'],
      ),
    'is_user_registered' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'receive_full_quiz_cache' : IDL.Func(
        [IDL.Vec(IDL.Tuple(IDL.Text, QuizCacheData))],
        [],
        [],
      ),
    'receive_quiz_cache' : IDL.Func([IDL.Text, QuizCacheData], [], []),
    'receive_token_limits' : IDL.Func([TokenLimitsConfig], [], []),
    'register_user' : IDL.Func(
        [UserProfileUpdate],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_archive_canister' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'submit_quiz' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8)],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'unstake' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'update_profile' : IDL.Func(
        [UserProfileUpdate],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'whoami' : IDL.Func([], [IDL.Principal], ['query']),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  return [InitArgs];
};
