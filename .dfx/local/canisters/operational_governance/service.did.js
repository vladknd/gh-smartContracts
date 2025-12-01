export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ledger_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
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
  return IDL.Service({
    'create_proposal' : IDL.Func(
        [IDL.Principal, IDL.Nat64, IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'execute_proposal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_proposal' : IDL.Func([IDL.Nat64], [IDL.Opt(Proposal)], ['query']),
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
