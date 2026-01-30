export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'price_per_token_e6' : IDL.Nat,
    'ghc_ledger_id' : IDL.Principal,
    'ghc_decimals' : IDL.Nat8,
    'treasury_principal' : IDL.Principal,
    'ckusdc_ledger_id' : IDL.Principal,
    'admin_principal' : IDL.Principal,
  });
  const IcoState = IDL.Record({
    'price_per_token_e6' : IDL.Nat,
    'total_sold_ghc' : IDL.Nat,
    'ghc_ledger_id' : IDL.Principal,
    'ghc_decimals' : IDL.Nat8,
    'total_raised_usdc' : IDL.Nat,
    'ckusdc_ledger_id' : IDL.Principal,
    'admin_principal' : IDL.Principal,
  });
  return IDL.Service({
    'buy_ghc' : IDL.Func(
        [IDL.Nat],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'end_sale' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'get_ico_stats' : IDL.Func([], [IcoState], ['query']),
    'withdraw_ghc' : IDL.Func(
        [IDL.Principal, IDL.Nat],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'withdraw_usdc' : IDL.Func(
        [IDL.Principal, IDL.Nat],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'price_per_token_e6' : IDL.Nat,
    'ghc_ledger_id' : IDL.Principal,
    'ghc_decimals' : IDL.Nat8,
    'treasury_principal' : IDL.Principal,
    'ckusdc_ledger_id' : IDL.Principal,
    'admin_principal' : IDL.Principal,
  });
  return [InitArgs];
};
