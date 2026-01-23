export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ledger_id' : IDL.Principal,
    'governance_canister_id' : IDL.Principal,
  });
  const TokenType = IDL.Variant({
    'GHC' : IDL.Null,
    'ICP' : IDL.Null,
    'USDC' : IDL.Null,
  });
  const ExecuteTransferInput = IDL.Record({
    'recipient' : IDL.Principal,
    'proposal_id' : IDL.Nat64,
    'amount' : IDL.Nat64,
    'token_type' : TokenType,
  });
  const MMCRStatus = IDL.Record({
    'last_release_timestamp' : IDL.Nat64,
    'releases_completed' : IDL.Nat64,
    'next_scheduled_year' : IDL.Nat16,
    'next_scheduled_month' : IDL.Nat8,
    'seconds_until_next' : IDL.Nat64,
    'next_release_amount' : IDL.Nat64,
    'releases_remaining' : IDL.Nat64,
  });
  const TreasuryState = IDL.Record({
    'balance' : IDL.Nat64,
    'total_transferred' : IDL.Nat64,
    'genesis_timestamp' : IDL.Nat64,
    'mmcr_count' : IDL.Nat64,
    'last_mmcr_year' : IDL.Nat16,
    'allowance' : IDL.Nat64,
    'last_mmcr_timestamp' : IDL.Nat64,
    'last_mmcr_month' : IDL.Nat8,
  });
  return IDL.Service({
    'can_execute_mmcr_now' : IDL.Func([], [IDL.Bool, IDL.Text], ['query']),
    'can_transfer' : IDL.Func([IDL.Nat64, TokenType], [IDL.Bool], ['query']),
    'execute_mmcr' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'execute_transfer' : IDL.Func(
        [ExecuteTransferInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'force_execute_mmcr' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'get_current_eastern_time' : IDL.Func(
        [],
        [IDL.Nat16, IDL.Nat8, IDL.Nat8, IDL.Nat8, IDL.Nat8, IDL.Nat8, IDL.Bool],
        ['query'],
      ),
    'get_governance_canister_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_ledger_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_mmcr_status' : IDL.Func([], [MMCRStatus], ['query']),
    'get_spendable_balance' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_test_timestamps_for_year' : IDL.Func(
        [IDL.Nat16],
        [IDL.Vec(IDL.Tuple(IDL.Nat8, IDL.Nat64, IDL.Bool))],
        ['query'],
      ),
    'get_treasury_balance' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_treasury_state' : IDL.Func([], [TreasuryState], ['query']),
    'reset_mmcr_for_testing' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'set_governance_canister_id' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'simulate_mmcr_at_time' : IDL.Func(
        [IDL.Nat64],
        [IDL.Bool, IDL.Text, IDL.Nat8, IDL.Nat16],
        ['query'],
      ),
    'test_date_parsing' : IDL.Func(
        [IDL.Nat64],
        [
          IDL.Nat16,
          IDL.Nat8,
          IDL.Nat8,
          IDL.Nat8,
          IDL.Nat8,
          IDL.Nat8,
          IDL.Bool,
          IDL.Nat8,
        ],
        ['query'],
      ),
    'test_dst_boundaries' : IDL.Func(
        [IDL.Nat16],
        [IDL.Nat8, IDL.Nat8, IDL.Nat8, IDL.Nat8],
        ['query'],
      ),
    'test_mmcr_window' : IDL.Func(
        [IDL.Nat16, IDL.Nat8, IDL.Nat8, IDL.Nat8, IDL.Nat8],
        [IDL.Bool, IDL.Nat64, IDL.Text],
        ['query'],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ledger_id' : IDL.Principal,
    'governance_canister_id' : IDL.Principal,
  });
  return [InitArgs];
};
