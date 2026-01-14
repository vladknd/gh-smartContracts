export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'ledger_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  const ProposalStatus = IDL.Variant({
    'Active' : IDL.Null,
    'Approved' : IDL.Null,
    'Rejected' : IDL.Null,
    'Proposed' : IDL.Null,
    'Executed' : IDL.Null,
  });
  const CreateBoardMemberProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'description' : IDL.Text,
    'new_member' : IDL.Principal,
    'percentage' : IDL.Nat8,
  });
  const ProposalCategory = IDL.Variant({
    'Partnership' : IDL.Null,
    'CommunityGrant' : IDL.Null,
    'Custom' : IDL.Text,
    'Operations' : IDL.Null,
    'Development' : IDL.Null,
    'Marketing' : IDL.Null,
    'Liquidity' : IDL.Null,
  });
  const TokenType = IDL.Variant({
    'GHC' : IDL.Null,
    'ICP' : IDL.Null,
    'USDC' : IDL.Null,
  });
  const CreateTreasuryProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'recipient' : IDL.Principal,
    'description' : IDL.Text,
    'category' : ProposalCategory,
    'amount' : IDL.Nat64,
    'token_type' : TokenType,
  });
  const AddBoardMemberPayload = IDL.Record({
    'new_member' : IDL.Principal,
    'percentage' : IDL.Nat8,
  });
  const ProposalType = IDL.Variant({
    'Treasury' : IDL.Null,
    'AddBoardMember' : IDL.Null,
  });
  const Proposal = IDL.Record({
    'id' : IDL.Nat64,
    'status' : ProposalStatus,
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'votes_no' : IDL.Nat64,
    'recipient' : IDL.Opt(IDL.Principal),
    'description' : IDL.Text,
    'created_at' : IDL.Nat64,
    'board_member_payload' : IDL.Opt(AddBoardMemberPayload),
    'voting_ends_at' : IDL.Nat64,
    'supporter_count' : IDL.Nat64,
    'category' : IDL.Opt(ProposalCategory),
    'proposer' : IDL.Principal,
    'voter_count' : IDL.Nat64,
    'votes_yes' : IDL.Nat64,
    'amount' : IDL.Opt(IDL.Nat64),
    'token_type' : IDL.Opt(TokenType),
    'proposal_type' : ProposalType,
    'support_amount' : IDL.Nat64,
  });
  const BoardMemberShare = IDL.Record({
    'member' : IDL.Principal,
    'percentage' : IDL.Nat8,
  });
  const MMCRStatus = IDL.Record({
    'last_release_timestamp' : IDL.Nat64,
    'releases_completed' : IDL.Nat64,
    'seconds_until_next' : IDL.Nat64,
    'next_release_amount' : IDL.Nat64,
    'releases_remaining' : IDL.Nat64,
  });
  const SupportRecord = IDL.Record({
    'supporter' : IDL.Principal,
    'proposal_id' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'support_amount' : IDL.Nat64,
  });
  const VoteRecord = IDL.Record({
    'voter' : IDL.Principal,
    'vote' : IDL.Bool,
    'proposal_id' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'voting_power' : IDL.Nat64,
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
    'admin_expire_proposal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'admin_set_proposal_status' : IDL.Func(
        [IDL.Nat64, ProposalStatus],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'are_board_shares_locked' : IDL.Func([], [IDL.Bool], ['query']),
    'create_board_member_proposal' : IDL.Func(
        [CreateBoardMemberProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_treasury_proposal' : IDL.Func(
        [CreateTreasuryProposalInput],
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
    'finalize_proposal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : ProposalStatus, 'Err' : IDL.Text })],
        [],
      ),
    'get_active_proposals' : IDL.Func([], [IDL.Vec(Proposal)], ['query']),
    'get_all_proposals' : IDL.Func([], [IDL.Vec(Proposal)], ['query']),
    'get_board_member_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_board_member_share' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(IDL.Nat8)],
        ['query'],
      ),
    'get_board_member_shares' : IDL.Func(
        [],
        [IDL.Vec(BoardMemberShare)],
        ['query'],
      ),
    'get_governance_config' : IDL.Func(
        [],
        [IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Nat64],
        ['query'],
      ),
    'get_mmcr_status' : IDL.Func([], [MMCRStatus], ['query']),
    'get_my_voting_power' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'get_proposal' : IDL.Func([IDL.Nat64], [IDL.Opt(Proposal)], ['query']),
    'get_proposal_supporters' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(SupportRecord)],
        ['query'],
      ),
    'get_proposal_votes' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(VoteRecord)],
        ['query'],
      ),
    'get_spendable_balance' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_treasury_state' : IDL.Func([], [TreasuryState], ['query']),
    'get_user_voting_power' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'has_voted' : IDL.Func([IDL.Nat64, IDL.Principal], [IDL.Bool], ['query']),
    'is_board_member' : IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    'lock_board_member_shares' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_board_member_shares' : IDL.Func(
        [IDL.Vec(BoardMemberShare)],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'support_proposal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
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
