import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface CreateProposalInput {
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
  'recipient' : Principal,
  'description' : string,
  'created_at' : bigint,
  'voting_ends_at' : bigint,
  'category' : ProposalCategory,
  'proposer' : Principal,
  'voter_count' : bigint,
  'votes_yes' : bigint,
  'amount' : bigint,
  'token_type' : TokenType,
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
  { 'Executed' : null };
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
  'create_proposal' : ActorMethod<
    [CreateProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'execute_mmcr' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'finalize_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : ProposalStatus } |
      { 'Err' : string }
  >,
  'get_active_proposals' : ActorMethod<[], Array<Proposal>>,
  'get_all_proposals' : ActorMethod<[], Array<Proposal>>,
  'get_governance_config' : ActorMethod<[], [bigint, bigint, bigint, bigint]>,
  'get_mmcr_status' : ActorMethod<[], MMCRStatus>,
  'get_proposal' : ActorMethod<[bigint], [] | [Proposal]>,
  'get_proposal_votes' : ActorMethod<[bigint], Array<VoteRecord>>,
  'get_spendable_balance' : ActorMethod<[], bigint>,
  'get_treasury_state' : ActorMethod<[], TreasuryState>,
  'get_usdc_ledger_id' : ActorMethod<[], Principal>,
  'has_voted' : ActorMethod<[bigint, Principal], boolean>,
  'set_usdc_ledger_id' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'vote' : ActorMethod<[bigint, boolean], { 'Ok' : null } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
