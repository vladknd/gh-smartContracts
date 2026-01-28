export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'ledger_id' : IDL.Principal });
  const VestingStatus = IDL.Record({
    'years_elapsed' : IDL.Nat64,
    'claimed' : IDL.Nat64,
    'claimable' : IDL.Nat64,
    'founder' : IDL.Principal,
    'vested' : IDL.Nat64,
    'total_allocation' : IDL.Nat64,
    'unlock_percentage' : IDL.Nat64,
  });
  return IDL.Service({
    'admin_claim_vested_at' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'admin_register_founder' : IDL.Func(
        [IDL.Principal, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'admin_set_genesis_timestamp' : IDL.Func([IDL.Nat64], [], []),
    'claim_vested' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'get_all_vesting_schedules' : IDL.Func(
        [],
        [IDL.Vec(VestingStatus)],
        ['query'],
      ),
    'get_genesis_timestamp' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_total_unclaimed' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_vesting_status' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(VestingStatus)],
        ['query'],
      ),
    'is_founder' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'ledger_id' : IDL.Principal });
  return [InitArgs];
};
