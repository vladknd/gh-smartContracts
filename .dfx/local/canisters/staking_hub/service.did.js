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
    'total_staked' : IDL.Nat64,
    'total_allocated' : IDL.Nat64,
    'total_unstaked' : IDL.Nat64,
  });
  return IDL.Service({
    'add_allowed_minter' : IDL.Func([IDL.Principal], [], []),
    'add_founder' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'ensure_capacity' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Opt(IDL.Principal), 'Err' : IDL.Text })],
        [],
      ),
    'fetch_voting_power' : IDL.Func([IDL.Principal], [IDL.Nat64], []),
    'get_active_shards' : IDL.Func([], [IDL.Vec(ShardInfo)], ['query']),
    'get_config' : IDL.Func(
        [],
        [IDL.Principal, IDL.Principal, IDL.Bool],
        ['query'],
      ),
    'get_founder_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_founders' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
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
    'is_founder' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'is_registered_shard' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
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
    'remove_founder' : IDL.Func(
        [IDL.Principal],
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
    'ledger_id' : IDL.Principal,
    'user_profile_wasm' : IDL.Vec(IDL.Nat8),
  });
  return [InitArgs];
};
