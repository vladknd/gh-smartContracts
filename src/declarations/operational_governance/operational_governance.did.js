export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ledger_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  const MMCRStatus = IDL.Record({
    'last_release_timestamp' : IDL.Nat64,
    'releases_completed' : IDL.Nat64,
    'seconds_until_next' : IDL.Nat64,
    'next_release_amount' : IDL.Nat64,
    'releases_remaining' : IDL.Nat64,
  });
  const Proposal = IDL.Record({
    'id' : IDL.Nat64,
    'votes_no' : IDL.Nat64,
    'recipient' : IDL.Principal,
    'description' : IDL.Text,
    'created_at' : IDL.Nat64,
    'proposer' : IDL.Principal,
    'votes_yes' : IDL.Nat64,
    'executed' : IDL.Bool,
    'amount' : IDL.Nat64,
  });
  const TreasuryState = IDL.Record({
    'balance' : IDL.Nat64,
    'total_transferred' : IDL.Nat64,
    'genesis_timestamp' : IDL.Nat64,
    'mmcr_count' : IDL.Nat64,
    'allowance' : IDL.Nat64,
    'last_mmcr_timestamp' : IDL.Nat64,
  });
  return IDL.Service({
    'create_proposal' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'execute_mmcr' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'execute_proposal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_mmcr_status' : IDL.Func([], [MMCRStatus], ['query']),
    'get_proposal' : IDL.Func([IDL.Nat64], [IDL.Opt(Proposal)], ['query']),
    'get_spendable_balance' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_total_spent' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_treasury_balance' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_treasury_state' : IDL.Func([], [TreasuryState], ['query']),
    'vote' : IDL.Func(
        [IDL.Nat64, IDL.Bool],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ledger_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  return [InitArgs];
};
