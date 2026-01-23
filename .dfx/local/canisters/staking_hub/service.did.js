export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'archive_canister_wasm' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'ledger_id' : IDL.Principal,
    'user_profile_wasm' : IDL.Vec(IDL.Nat8),
  });
  const QuizCacheData = IDL.Record({
    'question_count' : IDL.Nat8,
    'content_id' : IDL.Text,
    'version' : IDL.Nat64,
    'answer_hashes' : IDL.Vec(IDL.Vec(IDL.Nat8)),
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
  const ShardStatus = IDL.Variant({ 'Full' : IDL.Null, 'Active' : IDL.Null });
  const ShardInfo = IDL.Record({
    'user_count' : IDL.Nat64,
    'status' : ShardStatus,
    'canister_id' : IDL.Principal,
    'created_at' : IDL.Nat64,
    'archive_canister_id' : IDL.Opt(IDL.Principal),
  });
  const BoardMemberShare = IDL.Record({
    'member' : IDL.Principal,
    'percentage' : IDL.Nat8,
  });
  const GlobalStats = IDL.Record({
    'total_staked' : IDL.Nat64,
    'total_allocated' : IDL.Nat64,
    'total_unstaked' : IDL.Nat64,
  });
  return IDL.Service({
    'add_allowed_minter' : IDL.Func([IDL.Principal], [], []),
    'admin_set_user_shard' : IDL.Func(
        [IDL.Principal, IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'are_board_shares_locked' : IDL.Func([], [IDL.Bool], ['query']),
    'distribute_quiz_cache' : IDL.Func(
        [IDL.Text, QuizCacheData],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'distribute_quiz_config' : IDL.Func(
        [CachedQuizConfig],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'ensure_capacity' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Opt(IDL.Principal), 'Err' : IDL.Text })],
        [],
      ),
    'fetch_voting_power' : IDL.Func([IDL.Principal], [IDL.Nat64], []),
    'get_active_shards' : IDL.Func([], [IDL.Vec(ShardInfo)], ['query']),
    'get_archive_for_shard' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'get_board_member_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_board_member_share' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(IDL.Nat8)],
        ['query'],
      ),
    'get_board_member_shares' : IDL.Func(
        [],
        [IDL.Vec(BoardMemberShare)],
        ['query'],
      ),
    'get_config' : IDL.Func(
        [],
        [IDL.Principal, IDL.Principal, IDL.Bool],
        ['query'],
      ),
    'get_global_stats' : IDL.Func([], [GlobalStats], ['query']),
    'get_limits' : IDL.Func([], [IDL.Nat64, IDL.Nat64], ['query']),
    'get_shard_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_shard_for_new_user' : IDL.Func(
        [],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'get_shards' : IDL.Func([], [IDL.Vec(ShardInfo)], ['query']),
    'get_tokenomics' : IDL.Func(
        [],
        [IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Nat64],
        ['query'],
      ),
    'get_total_voting_power' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_user_shard' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'get_vuc' : IDL.Func([], [IDL.Nat64], ['query']),
    'is_board_member' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'is_registered_shard' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'lock_board_member_shares' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'process_unstake' : IDL.Func(
        [IDL.Principal, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'register_user_location' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_board_member_shares' : IDL.Func(
        [IDL.Vec(BoardMemberShare)],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'sync_shard' : IDL.Func(
        [IDL.Int64, IDL.Nat64, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'update_shard_user_count' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'archive_canister_wasm' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'ledger_id' : IDL.Principal,
    'user_profile_wasm' : IDL.Vec(IDL.Nat8),
  });
  return [InitArgs];
};
