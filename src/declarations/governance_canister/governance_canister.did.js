export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_engine_id' : IDL.Opt(IDL.Principal),
    'treasury_canister_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  const ProposalStatus = IDL.Variant({
    'Active' : IDL.Null,
    'Approved' : IDL.Null,
    'Rejected' : IDL.Null,
    'Proposed' : IDL.Null,
    'Executed' : IDL.Null,
  });
  const CreateAddContentProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'unit_count' : IDL.Nat32,
    'staging_path' : IDL.Text,
    'content_hash' : IDL.Text,
    'description' : IDL.Text,
    'staging_canister' : IDL.Principal,
    'content_title' : IDL.Text,
  });
  const CreateBoardMemberProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'description' : IDL.Text,
    'new_member' : IDL.Principal,
    'percentage' : IDL.Nat8,
  });
  const CreateDeleteContentProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'content_id' : IDL.Text,
    'description' : IDL.Text,
    'reason' : IDL.Text,
  });
  const CreateRemoveBoardMemberProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'description' : IDL.Text,
    'member_to_remove' : IDL.Principal,
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
  const CreateUpdateBoardMemberShareProposalInput = IDL.Record({
    'member' : IDL.Principal,
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'description' : IDL.Text,
    'new_percentage' : IDL.Nat8,
  });
  const CreateUpdateGovernanceConfigProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'new_support_period_days' : IDL.Opt(IDL.Nat16),
    'new_approval_percentage' : IDL.Opt(IDL.Nat8),
    'description' : IDL.Text,
    'new_min_voting_power' : IDL.Opt(IDL.Nat64),
    'new_support_threshold' : IDL.Opt(IDL.Nat64),
    'new_voting_period_days' : IDL.Opt(IDL.Nat16),
    'new_resubmission_cooldown_days' : IDL.Opt(IDL.Nat16),
  });
  const CreateUpdateQuizConfigProposalInput = IDL.Record({
    'external_link' : IDL.Opt(IDL.Text),
    'new_pass_threshold' : IDL.Opt(IDL.Nat8),
    'title' : IDL.Text,
    'new_max_daily_quizzes' : IDL.Opt(IDL.Nat8),
    'new_max_weekly_quizzes' : IDL.Opt(IDL.Nat8),
    'new_max_yearly_quizzes' : IDL.Opt(IDL.Nat16),
    'new_max_attempts' : IDL.Opt(IDL.Nat8),
    'description' : IDL.Text,
    'new_max_monthly_quizzes' : IDL.Opt(IDL.Nat8),
    'new_reward_amount' : IDL.Opt(IDL.Nat64),
  });
  const UpdateGlobalQuizConfigPayload = IDL.Record({
    'new_pass_threshold' : IDL.Opt(IDL.Nat8),
    'new_max_daily_quizzes' : IDL.Opt(IDL.Nat8),
    'new_max_weekly_quizzes' : IDL.Opt(IDL.Nat8),
    'new_max_yearly_quizzes' : IDL.Opt(IDL.Nat16),
    'new_max_attempts' : IDL.Opt(IDL.Nat8),
    'new_max_monthly_quizzes' : IDL.Opt(IDL.Nat8),
    'new_reward_amount' : IDL.Opt(IDL.Nat64),
  });
  const AddContentFromStagingPayload = IDL.Record({
    'unit_count' : IDL.Nat32,
    'staging_path' : IDL.Text,
    'content_hash' : IDL.Text,
    'staging_canister' : IDL.Principal,
    'content_title' : IDL.Text,
  });
  const QuizQuestion = IDL.Record({
    'question' : IDL.Text,
    'answer' : IDL.Nat8,
    'options' : IDL.Vec(IDL.Text),
  });
  const UpdateQuizQuestionsPayload = IDL.Record({
    'content_id' : IDL.Text,
    'new_questions' : IDL.Vec(QuizQuestion),
  });
  const DeleteContentNodePayload = IDL.Record({
    'content_id' : IDL.Text,
    'reason' : IDL.Text,
  });
  const AddBoardMemberPayload = IDL.Record({
    'new_member' : IDL.Principal,
    'percentage' : IDL.Nat8,
  });
  const UpdateGovernanceConfigPayload = IDL.Record({
    'new_support_period_days' : IDL.Opt(IDL.Nat16),
    'new_approval_percentage' : IDL.Opt(IDL.Nat8),
    'new_min_voting_power' : IDL.Opt(IDL.Nat64),
    'new_support_threshold' : IDL.Opt(IDL.Nat64),
    'new_voting_period_days' : IDL.Opt(IDL.Nat16),
    'new_resubmission_cooldown_days' : IDL.Opt(IDL.Nat16),
  });
  const RemoveBoardMemberPayload = IDL.Record({
    'member_to_remove' : IDL.Principal,
  });
  const UpdateContentNodePayload = IDL.Record({
    'content_id' : IDL.Text,
    'new_title' : IDL.Opt(IDL.Text),
    'new_content' : IDL.Opt(IDL.Text),
    'new_paraphrase' : IDL.Opt(IDL.Text),
  });
  const UpdateBoardMemberSharePayload = IDL.Record({
    'member' : IDL.Principal,
    'new_percentage' : IDL.Nat8,
  });
  const ProposalType = IDL.Variant({
    'UpdateGovernanceConfig' : IDL.Null,
    'UpdateBoardMemberShare' : IDL.Null,
    'UpdateContentNode' : IDL.Null,
    'UpdateGlobalQuizConfig' : IDL.Null,
    'AddContentFromStaging' : IDL.Null,
    'UpdateQuizQuestions' : IDL.Null,
    'DeleteContentNode' : IDL.Null,
    'Treasury' : IDL.Null,
    'AddBoardMember' : IDL.Null,
    'RemoveBoardMember' : IDL.Null,
  });
  const Proposal = IDL.Record({
    'id' : IDL.Nat64,
    'status' : ProposalStatus,
    'external_link' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'update_quiz_config_payload' : IDL.Opt(UpdateGlobalQuizConfigPayload),
    'add_content_payload' : IDL.Opt(AddContentFromStagingPayload),
    'votes_no' : IDL.Nat64,
    'recipient' : IDL.Opt(IDL.Principal),
    'update_quiz_questions_payload' : IDL.Opt(UpdateQuizQuestionsPayload),
    'description' : IDL.Text,
    'delete_content_payload' : IDL.Opt(DeleteContentNodePayload),
    'created_at' : IDL.Nat64,
    'board_member_payload' : IDL.Opt(AddBoardMemberPayload),
    'update_governance_config_payload' : IDL.Opt(UpdateGovernanceConfigPayload),
    'required_yes_votes' : IDL.Nat64,
    'voting_ends_at' : IDL.Nat64,
    'supporter_count' : IDL.Nat64,
    'category' : IDL.Opt(ProposalCategory),
    'proposer' : IDL.Principal,
    'voter_count' : IDL.Nat64,
    'votes_yes' : IDL.Nat64,
    'remove_board_member_payload' : IDL.Opt(RemoveBoardMemberPayload),
    'update_content_payload' : IDL.Opt(UpdateContentNodePayload),
    'update_board_member_payload' : IDL.Opt(UpdateBoardMemberSharePayload),
    'amount' : IDL.Opt(IDL.Nat64),
    'token_type' : IDL.Opt(TokenType),
    'proposal_type' : ProposalType,
    'support_amount' : IDL.Nat64,
  });
  const BoardMemberShare = IDL.Record({
    'member' : IDL.Principal,
    'percentage' : IDL.Nat8,
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
    'create_add_content_proposal' : IDL.Func(
        [CreateAddContentProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_board_member_proposal' : IDL.Func(
        [CreateBoardMemberProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_delete_content_proposal' : IDL.Func(
        [CreateDeleteContentProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_remove_board_member_proposal' : IDL.Func(
        [CreateRemoveBoardMemberProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_treasury_proposal' : IDL.Func(
        [CreateTreasuryProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_update_board_member_share_proposal' : IDL.Func(
        [CreateUpdateBoardMemberShareProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_update_governance_config_proposal' : IDL.Func(
        [CreateUpdateGovernanceConfigProposalInput],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'create_update_quiz_config_proposal' : IDL.Func(
        [CreateUpdateQuizConfigProposalInput],
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
        [IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Nat8],
        ['query'],
      ),
    'get_learning_engine_id' : IDL.Func([], [IDL.Principal], ['query']),
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
    'get_staking_hub_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_treasury_canister_id' : IDL.Func([], [IDL.Principal], ['query']),
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
    'set_learning_engine_id' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_treasury_canister_id' : IDL.Func(
        [IDL.Principal],
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
    'learning_engine_id' : IDL.Opt(IDL.Principal),
    'treasury_canister_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  return [InitArgs];
};
