import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AddBoardMemberPayload {
  'share_bps' : number,
  'new_member' : Principal,
}
export interface AddContentFromStagingPayload {
  'unit_count' : number,
  'staging_path' : string,
  'content_hash' : string,
  'staging_canister' : Principal,
  'content_title' : string,
}
export interface BoardMemberShare {
  'member' : Principal,
  'is_sentinel' : boolean,
  'share_bps' : number,
}
export interface CreateAddContentProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'unit_count' : number,
  'staging_path' : string,
  'content_hash' : string,
  'description' : string,
  'staging_canister' : Principal,
  'content_title' : string,
}
export interface CreateBoardMemberProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'description' : string,
  'share_bps' : number,
  'new_member' : Principal,
}
export interface CreateDeleteContentProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'content_id' : string,
  'description' : string,
  'reason' : string,
}
export interface CreateRemoveBoardMemberProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'description' : string,
  'member_to_remove' : Principal,
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
export interface CreateUpdateBoardMemberShareProposalInput {
  'member' : Principal,
  'external_link' : [] | [string],
  'title' : string,
  'description' : string,
  'new_share_bps' : number,
}
export interface CreateUpdateGovernanceConfigProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'new_support_period_days' : [] | [number],
  'new_approval_percentage' : [] | [number],
  'description' : string,
  'new_min_voting_power' : [] | [bigint],
  'new_support_threshold' : [] | [bigint],
  'new_voting_period_days' : [] | [number],
  'new_resubmission_cooldown_days' : [] | [number],
}
export interface CreateUpdateSentinelProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  'description' : string,
  'new_sentinel' : Principal,
}
export interface CreateUpdateTokenLimitsProposalInput {
  'external_link' : [] | [string],
  'new_pass_threshold' : [] | [number],
  'title' : string,
  'new_max_attempts' : [] | [number],
  'description' : string,
  'new_reward_amount' : [] | [bigint],
  'new_regular_limits' : [] | [TokenLimits],
  'new_subscribed_limits' : [] | [TokenLimits],
}
export interface DeleteContentNodePayload {
  'content_id' : string,
  'reason' : string,
}
export interface InitArgs {
  'learning_engine_id' : [] | [Principal],
  'treasury_canister_id' : Principal,
  'staking_hub_id' : Principal,
}
export interface Proposal {
  'id' : bigint,
  'status' : ProposalStatus,
  'external_link' : [] | [string],
  'title' : string,
  'add_content_payload' : [] | [AddContentFromStagingPayload],
  'votes_no' : bigint,
  'recipient' : [] | [Principal],
  'description' : string,
  'delete_content_payload' : [] | [DeleteContentNodePayload],
  'created_at' : bigint,
  'board_member_payload' : [] | [AddBoardMemberPayload],
  'update_governance_config_payload' : [] | [UpdateGovernanceConfigPayload],
  'required_yes_votes' : bigint,
  'voting_ends_at' : bigint,
  'supporter_count' : bigint,
  'update_sentinel_payload' : [] | [UpdateSentinelPayload],
  'category' : [] | [ProposalCategory],
  'proposer' : Principal,
  'voter_count' : bigint,
  'votes_yes' : bigint,
  'remove_board_member_payload' : [] | [RemoveBoardMemberPayload],
  'update_token_limits_payload' : [] | [UpdateTokenLimitsPayload],
  'update_board_member_payload' : [] | [UpdateBoardMemberSharePayload],
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
export type ProposalType = { 'UpdateSentinel' : null } |
  { 'UpdateGovernanceConfig' : null } |
  { 'UpdateBoardMemberShare' : null } |
  { 'AddContentFromStaging' : null } |
  { 'UpdateTokenLimits' : null } |
  { 'DeleteContentNode' : null } |
  { 'Treasury' : null } |
  { 'AddBoardMember' : null } |
  { 'RemoveBoardMember' : null };
export interface RemoveBoardMemberPayload { 'member_to_remove' : Principal }
export interface SupportRecord {
  'supporter' : Principal,
  'proposal_id' : bigint,
  'timestamp' : bigint,
  'support_amount' : bigint,
}
export interface TokenLimits {
  'max_monthly_tokens' : bigint,
  'max_yearly_tokens' : bigint,
  'max_daily_tokens' : bigint,
  'max_weekly_tokens' : bigint,
}
export type TokenType = { 'GHC' : null } |
  { 'ICP' : null } |
  { 'USDC' : null };
export interface UpdateBoardMemberSharePayload {
  'member' : Principal,
  'new_share_bps' : number,
}
export interface UpdateGovernanceConfigPayload {
  'new_support_period_days' : [] | [number],
  'new_approval_percentage' : [] | [number],
  'new_min_voting_power' : [] | [bigint],
  'new_support_threshold' : [] | [bigint],
  'new_voting_period_days' : [] | [number],
  'new_resubmission_cooldown_days' : [] | [number],
}
export interface UpdateSentinelPayload { 'new_sentinel' : Principal }
export interface UpdateTokenLimitsPayload {
  'new_pass_threshold' : [] | [number],
  'new_max_attempts' : [] | [number],
  'new_reward_amount' : [] | [bigint],
  'new_regular_limits' : [] | [TokenLimits],
  'new_subscribed_limits' : [] | [TokenLimits],
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
  'clear_sentinel_member' : ActorMethod<
    [],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'create_add_content_proposal' : ActorMethod<
    [CreateAddContentProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_board_member_proposal' : ActorMethod<
    [CreateBoardMemberProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_delete_content_proposal' : ActorMethod<
    [CreateDeleteContentProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_remove_board_member_proposal' : ActorMethod<
    [CreateRemoveBoardMemberProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_treasury_proposal' : ActorMethod<
    [CreateTreasuryProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_update_board_member_share_proposal' : ActorMethod<
    [CreateUpdateBoardMemberShareProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_update_governance_config_proposal' : ActorMethod<
    [CreateUpdateGovernanceConfigProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_update_sentinel_proposal' : ActorMethod<
    [CreateUpdateSentinelProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'create_update_token_limits_proposal' : ActorMethod<
    [CreateUpdateTokenLimitsProposalInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
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
  'get_all_board_member_voting_powers' : ActorMethod<
    [],
    { 'Ok' : Array<[Principal, number, bigint, boolean]> } |
      { 'Err' : string }
  >,
  'get_all_proposals' : ActorMethod<[], Array<Proposal>>,
  'get_board_member_count' : ActorMethod<[], bigint>,
  'get_board_member_share' : ActorMethod<[Principal], [] | [number]>,
  'get_board_member_shares' : ActorMethod<[], Array<BoardMemberShare>>,
  'get_governance_config' : ActorMethod<
    [],
    [bigint, bigint, bigint, bigint, bigint, number]
  >,
  'get_learning_engine_id' : ActorMethod<[], Principal>,
  'get_my_voting_power' : ActorMethod<
    [],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'get_proposal' : ActorMethod<[bigint], [] | [Proposal]>,
  'get_proposal_supporters' : ActorMethod<[bigint], Array<SupportRecord>>,
  'get_proposal_votes' : ActorMethod<[bigint], Array<VoteRecord>>,
  'get_sentinel_member' : ActorMethod<[], [] | [Principal]>,
  'get_staking_hub_id' : ActorMethod<[], Principal>,
  'get_treasury_canister_id' : ActorMethod<[], Principal>,
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
  'set_learning_engine_id' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_sentinel_member' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_treasury_canister_id' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'support_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'unlock_board_member_shares' : ActorMethod<
    [],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'vote' : ActorMethod<[bigint, boolean], { 'Ok' : null } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
