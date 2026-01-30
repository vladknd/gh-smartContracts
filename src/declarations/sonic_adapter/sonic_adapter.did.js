export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'owner' : IDL.Principal,
    'ghc_ledger_id' : IDL.Principal,
    'usdc_ledger_id' : IDL.Principal,
    'sonic_canister_id' : IDL.Principal,
  });
  const LaunchIcoArgs = IDL.Record({
    'usdc_amount' : IDL.Nat64,
    'ghc_amount' : IDL.Nat64,
  });
  return IDL.Service({
    'add_liquidity' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Nat64, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'launch_ico' : IDL.Func(
        [LaunchIcoArgs],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'swap' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Principal, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'owner' : IDL.Principal,
    'ghc_ledger_id' : IDL.Principal,
    'usdc_ledger_id' : IDL.Principal,
    'sonic_canister_id' : IDL.Principal,
  });
  return [InitArgs];
};
