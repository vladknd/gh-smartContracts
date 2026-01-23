import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AddBoardMemberPayload {
  'new_member' : Principal,
  'percentage' : number,
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
  'percentage' : number,
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
  'new_member' : Principal,
  'percentage' : number,
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
/**
 * ============================================================================
 * Proposal Creation Input Types
 * ============================================================================
 */
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
  'new_percentage' : number,
}
export interface CreateUpdateGovernanceConfigProposalInput {
  'external_link' : [] | [string],
  'title' : string,
  /**
   * Timing Configuration (in days, 1-365)
   */
  'new_support_period_days' : [] | [number],
  'new_approval_percentage' : [] | [number],
  'description' : string,
  'new_min_voting_power' : [] | [bigint],
  'new_support_threshold' : [] | [bigint],
  'new_voting_period_days' : [] | [number],
  'new_resubmission_cooldown_days' : [] | [number],
}
export interface CreateUpdateQuizConfigProposalInput {
  'external_link' : [] | [string],
  'new_pass_threshold' : [] | [number],
  'title' : string,
  'new_max_daily_quizzes' : [] | [number],
  'new_max_weekly_quizzes' : [] | [number],
  'new_max_yearly_quizzes' : [] | [number],
  'new_max_attempts' : [] | [number],
  'description' : string,
  'new_max_monthly_quizzes' : [] | [number],
  'new_reward_amount' : [] | [bigint],
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
  'update_quiz_config_payload' : [] | [UpdateGlobalQuizConfigPayload],
  'add_content_payload' : [] | [AddContentFromStagingPayload],
  'votes_no' : bigint,
  'recipient' : [] | [Principal],
  'update_quiz_questions_payload' : [] | [UpdateQuizQuestionsPayload],
  'description' : string,
  'delete_content_payload' : [] | [DeleteContentNodePayload],
  'created_at' : bigint,
  'board_member_payload' : [] | [AddBoardMemberPayload],
  'update_governance_config_payload' : [] | [UpdateGovernanceConfigPayload],
  'required_yes_votes' : bigint,
  'voting_ends_at' : bigint,
  'supporter_count' : bigint,
  'category' : [] | [ProposalCategory],
  'proposer' : Principal,
  'voter_count' : bigint,
  'votes_yes' : bigint,
  'remove_board_member_payload' : [] | [RemoveBoardMemberPayload],
  'update_content_payload' : [] | [UpdateContentNodePayload],
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
export type ProposalType = { 'UpdateGovernanceConfig' : null } |
  { 'UpdateBoardMemberShare' : null } |
  { 'UpdateContentNode' : null } |
  { 'UpdateGlobalQuizConfig' : null } |
  { 'AddContentFromStaging' : null } |
  { 'UpdateQuizQuestions' : null } |
  { 'DeleteContentNode' : null } |
  { 'Treasury' : null } |
  { 'AddBoardMember' : null } |
  { 'RemoveBoardMember' : null };
/**
 * ============================================================================
 * Content Governance Payloads
 * ============================================================================
 */
export interface QuizQuestion {
  'question' : string,
  'answer' : number,
  'options' : Array<string>,
}
export interface RemoveBoardMemberPayload { 'member_to_remove' : Principal }
export interface SupportRecord {
  'supporter' : Principal,
  'proposal_id' : bigint,
  'timestamp' : bigint,
  'support_amount' : bigint,
}
/**
 * ============================================================================
 * Governance Canister Candid Interface
 * ============================================================================
 * Proposal lifecycle, voting, board member management, and content governance
 * Updated: January 2026
 */
export type TokenType = { 'GHC' : null } |
  { 'ICP' : null } |
  { 'USDC' : null };
export interface UpdateBoardMemberSharePayload {
  'member' : Principal,
  'new_percentage' : number,
}
export interface UpdateContentNodePayload {
  'content_id' : string,
  'new_title' : [] | [string],
  'new_content' : [] | [string],
  'new_paraphrase' : [] | [string],
}
export interface UpdateGlobalQuizConfigPayload {
  'new_pass_threshold' : [] | [number],
  'new_max_daily_quizzes' : [] | [number],
  'new_max_weekly_quizzes' : [] | [number],
  'new_max_yearly_quizzes' : [] | [number],
  'new_max_attempts' : [] | [number],
  'new_max_monthly_quizzes' : [] | [number],
  'new_reward_amount' : [] | [bigint],
}
export interface UpdateGovernanceConfigPayload {
  /**
   * Timing Configuration
   */
  'new_support_period_days' : [] | [number],
  'new_approval_percentage' : [] | [number],
  'new_min_voting_power' : [] | [bigint],
  'new_support_threshold' : [] | [bigint],
  'new_voting_period_days' : [] | [number],
  'new_resubmission_cooldown_days' : [] | [number],
}
export interface UpdateQuizQuestionsPayload {
  'content_id' : string,
  'new_questions' : Array<QuizQuestion>,
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
  'create_update_quiz_config_proposal' : ActorMethod<
    [CreateUpdateQuizConfigProposalInput],
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
  'vote' : ActorMethod<[bigint, boolean], { 'Ok' : null } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
