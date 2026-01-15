import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AddBoardMemberPayload {
  'new_member' : Principal,
  'percentage' : number,
}
export interface BoardMemberShare {
  'member' : Principal,
  'percentage' : number,
}
export interface CreateBoardMemberProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'description' : string,
  'new_member' : Principal,
  'percentage' : number,
}
export interface CreateTreasuryProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'recipient' : Principal,
  'description' : string,
  'category' : ProposalCategory,
  'amount' : bigint,
  'token_type' : TokenType,
}
export interface InitArgs {
  'ledger_id' : Principal,
  'staking_hub_id' : Principal,
}
export interface MMCRStatus {
  'last_release_timestamp' : bigint,
  'releases_completed' : bigint,
  'seconds_until_next' : bigint,
  'next_release_amount' : bigint,
  'releases_remaining' : bigint,
}
export interface Proposal {
  'id' : bigint,
  'status' : ProposalStatus,
  'external_link' : [] | [string],
  'title' : string,
  'votes_no' : bigint,
  'recipient' : [] | [Principal],
  'description' : string,
  'created_at' : bigint,
  'board_member_payload' : [] | [AddBoardMemberPayload],
  'voting_ends_at' : bigint,
  'supporter_count' : bigint,
  'category' : [] | [ProposalCategory],
  'proposer' : Principal,
  'voter_count' : bigint,
  'votes_yes' : bigint,
  'amount' : [] | [bigint],
  'token_type' : [] | [TokenType],
  'proposal_type' : ProposalType,
  'support_amount' : bigint,
}
export type ProposalCategory = { 'Partnership' : null } |
  { 'CommunityGrant' : null } |
  { 'Custom' : string } |
  { 'Operations' : null } |
  { 'Development' : null } |
  { 'Marketing' : null } |
  { 'Liquidity' : null };
export type ProposalStatus = { 'Active' : null } |
  { 'Approved' : null } |
  { 'Rejected' : null } |
  { 'Proposed' : null } |
  { 'Executed' : null };
export type ProposalType = { 'Treasury' : null } |
  { 'AddBoardMember' : null };
export interface SupportRecord {
  'supporter' : Principal,
  'proposal_id' : bigint,
  'timestamp' : bigint,
  'support_amount' : bigint,
}
export type TokenType = { 'GHC' : null } |
  { 'ICP' : null } |
  { 'USDC' : null };
export interface TreasuryState {
  'balance' : bigint,
  'total_transferred' : bigint,
  'genesis_timestamp' : bigint,
  'mmcr_count' : bigint,
  'allowance' : bigint,
  'last_mmcr_timestamp' : bigint,
}
export interface VoteRecord {
  'voter' : Principal,
  'vote' : boolean,
  'proposal_id' : bigint,
  'timestamp' : bigint,
  'voting_power' : bigint,
}
export interface _SERVICE {
  'admin_expire_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'admin_set_proposal_status' : ActorMethod<
    [bigint, ProposalStatus],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'are_board_shares_locked' : ActorMethod<[], boolean>,
  'create_board_member_proposal' : ActorMethod<
    [CreateBoardMemberProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_treasury_proposal' : ActorMethod<
    [CreateTreasuryProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'execute_mmcr' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'execute_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'finalize_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : ProposalStatus } |
      { 'Err' : string }
  >,
  'get_active_proposals' : ActorMethod<[], Array<Proposal>>,
  'get_all_proposals' : ActorMethod<[], Array<Proposal>>,
  'get_board_member_count' : ActorMethod<[], bigint>,
  'get_board_member_share' : ActorMethod<[Principal], [] | [number]>,
  'get_board_member_shares' : ActorMethod<[], Array<BoardMemberShare>>,
  'get_governance_config' : ActorMethod<
    [],
    [bigint, bigint, bigint, bigint, bigint]
  >,
  'get_mmcr_status' : ActorMethod<[], MMCRStatus>,
  'get_my_voting_power' : ActorMethod<
    [],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'get_proposal' : ActorMethod<[bigint], [] | [Proposal]>,
  'get_proposal_supporters' : ActorMethod<[bigint], Array<SupportRecord>>,
  'get_proposal_votes' : ActorMethod<[bigint], Array<VoteRecord>>,
  'get_spendable_balance' : ActorMethod<[], bigint>,
  'get_treasury_state' : ActorMethod<[], TreasuryState>,
  'get_user_voting_power' : ActorMethod<
    [Principal],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'has_voted' : ActorMethod<[bigint, Principal], boolean>,
  'is_board_member' : ActorMethod<[Principal], boolean>,
  'lock_board_member_shares' : ActorMethod<
    [],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_board_member_shares' : ActorMethod<
    [Array<BoardMemberShare>],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'support_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'vote' : ActorMethod<[bigint, boolean], { 'Ok' : null } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
