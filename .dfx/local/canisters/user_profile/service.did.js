export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  const UserProfile = IDL.Record({
    'name' : IDL.Text,
    'education' : IDL.Text,
    'email' : IDL.Text,
    'staked_balance' : IDL.Nat64,
    'gender' : IDL.Text,
    'transaction_count' : IDL.Nat64,
  });
  const VerificationTier = IDL.Variant({
    'KYC' : IDL.Null,
    'None' : IDL.Null,
    'Human' : IDL.Null,
  });
  const UserSummary = IDL.Record({
    'user_principal' : IDL.Principal,
    'name' : IDL.Text,
    'email' : IDL.Text,
    'staked_balance' : IDL.Nat64,
    'verification_tier' : VerificationTier,
  });
  const UserListResult = IDL.Record({
    'page_size' : IDL.Nat32,
    'page' : IDL.Nat32,
    'users' : IDL.Vec(UserSummary),
    'total_count' : IDL.Nat64,
    'has_more' : IDL.Bool,
  });
  const ArchiveConfig = IDL.Record({
    'trigger_threshold' : IDL.Nat64,
    'is_configured' : IDL.Bool,
    'archive_canister_id' : IDL.Principal,
    'retention_limit' : IDL.Nat64,
    'check_interval_secs' : IDL.Nat64,
  });
  const CachedQuizConfig = IDL.Record({
    'max_daily_quizzes' : IDL.Nat8,
    'reward_amount' : IDL.Nat64,
    'max_monthly_quizzes' : IDL.Nat8,
    'pass_threshold_percent' : IDL.Nat8,
    'max_daily_attempts' : IDL.Nat8,
    'version' : IDL.Nat64,
    'max_weekly_quizzes' : IDL.Nat8,
    'max_yearly_quizzes' : IDL.Nat16,
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
    'source' : IDL.Text,
    'archive_canister_id' : IDL.Principal,
    'transactions' : IDL.Vec(TransactionRecord),
    'archived_count' : IDL.Nat64,
    'local_count' : IDL.Nat64,
    'total_count' : IDL.Nat64,
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
    'debug_force_sync' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_archive_canister' : IDL.Func([], [IDL.Principal], ['query']),
    'get_archive_config' : IDL.Func([], [ArchiveConfig], ['query']),
    'get_cached_quiz_config' : IDL.Func([], [CachedQuizConfig], ['query']),
    'get_profile' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(UserProfile)],
        ['query'],
      ),
    'get_transactions_page' : IDL.Func(
        [IDL.Principal, IDL.Nat32],
        [TransactionPage],
        ['query'],
      ),
    'get_user_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_user_daily_status' : IDL.Func(
        [IDL.Principal],
        [UserTimeStats],
        ['query'],
      ),
    'get_user_stats' : IDL.Func([IDL.Principal], [UserTimeStats], ['query']),
    'get_user_transactions' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(TransactionRecord)],
        ['query'],
      ),
    'is_quiz_completed' : IDL.Func(
        [IDL.Principal, IDL.Text],
        [IDL.Bool],
        ['query'],
      ),
    'is_user_registered' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'receive_quiz_config' : IDL.Func([CachedQuizConfig], [], []),
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
    'trigger_archive' : IDL.Func(
        [],
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
