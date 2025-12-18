export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'ledger_id' : IDL.Principal,
    'user_profile_wasm' : IDL.Vec(IDL.Nat8),
  });
  const ShardStatus = IDL.Variant({ 'Full' : IDL.Null, 'Active' : IDL.Null });
  const ShardInfo = IDL.Record({
    'user_count' : IDL.Nat64,
    'status' : ShardStatus,
    'canister_id' : IDL.Principal,
    'created_at' : IDL.Nat64,
  });
  const GlobalStats = IDL.Record({
    'cumulative_reward_index' : IDL.Nat,
    'tier_reward_indexes' : IDL.Vec(IDL.Nat),
    'total_staked' : IDL.Nat64,
    'total_allocated' : IDL.Nat64,
    'tier_staked' : IDL.Vec(IDL.Nat64),
    'total_unstaked' : IDL.Nat64,
    'interest_pool' : IDL.Nat64,
    'total_rewards_distributed' : IDL.Nat64,
  });
  const TierDeltas = IDL.Vec(IDL.Int64);
  const TierIndexes = IDL.Vec(IDL.Nat);
  return IDL.Service({
    'add_allowed_minter' : IDL.Func([IDL.Principal], [], []),
    'distribute_interest' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'ensure_capacity' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Opt(IDL.Principal), 'Err' : IDL.Text })],
        [],
      ),
    'get_active_shards' : IDL.Func([], [IDL.Vec(ShardInfo)], ['query']),
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
    'is_registered_shard' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'process_unstake' : IDL.Func(
        [IDL.Principal, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'sync_shard' : IDL.Func(
        [TierDeltas, IDL.Nat64, IDL.Nat64, IDL.Nat64],
        [
          IDL.Variant({
            'Ok' : IDL.Tuple(IDL.Nat64, TierIndexes),
            'Err' : IDL.Text,
          }),
        ],
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
    'ledger_id' : IDL.Principal,
    'user_profile_wasm' : IDL.Vec(IDL.Nat8),
  });
  return [InitArgs];
};
