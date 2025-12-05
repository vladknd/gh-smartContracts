export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'ledger_id' : IDL.Principal });
  const GlobalStats = IDL.Record({
    'cumulative_reward_index' : IDL.Nat,
    'total_staked' : IDL.Nat64,
    'total_allocated' : IDL.Nat64,
    'total_unstaked' : IDL.Nat64,
    'interest_pool' : IDL.Nat64,
    'total_rewards_distributed' : IDL.Nat64,
  });
  return IDL.Service({
    'add_allowed_minter' : IDL.Func([IDL.Principal], [], []),
    'distribute_interest' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'get_global_stats' : IDL.Func([], [GlobalStats], ['query']),
    'process_unstake' : IDL.Func(
        [IDL.Principal, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'remove_allowed_minter' : IDL.Func([IDL.Principal], [], []),
    'sync_shard' : IDL.Func(
        [IDL.Int64, IDL.Nat64, IDL.Nat64, IDL.Nat64],
        [
          IDL.Variant({
            'Ok' : IDL.Tuple(IDL.Nat64, IDL.Nat),
            'Err' : IDL.Text,
          }),
        ],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'ledger_id' : IDL.Principal });
  return [InitArgs];
};
