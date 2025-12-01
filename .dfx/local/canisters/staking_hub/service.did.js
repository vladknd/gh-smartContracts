export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'ledger_id' : IDL.Principal });
  const GlobalStats = IDL.Record({
    'cumulative_reward_index' : IDL.Nat,
    'total_staked' : IDL.Nat64,
    'interest_pool' : IDL.Nat64,
  });
  return IDL.Service({
    'distribute_interest' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'get_global_stats' : IDL.Func([], [GlobalStats], ['query']),
    'get_user_stats' : IDL.Func(
        [IDL.Principal],
        [IDL.Nat64, IDL.Nat64],
        ['query'],
      ),
    'get_voting_power' : IDL.Func([IDL.Principal], [IDL.Nat64], ['query']),
    'stake_rewards' : IDL.Func([IDL.Principal, IDL.Nat64], [], []),
    'unstake' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'ledger_id' : IDL.Principal });
  return [InitArgs];
};
