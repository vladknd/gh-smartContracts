export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_engine_id' : IDL.Principal,
    'governance_canister_id' : IDL.Principal,
  });
  const MediaType = IDL.Variant({
    'PDF' : IDL.Null,
    'Image' : IDL.Null,
    'Audio' : IDL.Null,
    'Video' : IDL.Null,
  });
  const MediaContent = IDL.Record({
    'url' : IDL.Text,
    'duration_seconds' : IDL.Opt(IDL.Nat32),
    'media_type' : MediaType,
    'file_hash' : IDL.Opt(IDL.Text),
    'thumbnail_url' : IDL.Opt(IDL.Text),
  });
  const QuizQuestion = IDL.Record({
    'question' : IDL.Text,
    'answer' : IDL.Nat8,
    'options' : IDL.Vec(IDL.Text),
  });
  const QuizData = IDL.Record({ 'questions' : IDL.Vec(QuizQuestion) });
  const ContentNode = IDL.Record({
    'id' : IDL.Text,
    'media' : IDL.Opt(MediaContent),
    'title' : IDL.Text,
    'updated_at' : IDL.Nat64,
    'content' : IDL.Opt(IDL.Text),
    'order' : IDL.Nat32,
    'paraphrase' : IDL.Opt(IDL.Text),
    'quiz' : IDL.Opt(QuizData),
    'description' : IDL.Opt(IDL.Text),
    'created_at' : IDL.Nat64,
    'display_type' : IDL.Text,
    'version' : IDL.Nat64,
    'parent_id' : IDL.Opt(IDL.Text),
  });
  const StagingStatus = IDL.Variant({
    'ProposalCreated' : IDL.Null,
    'Rejected' : IDL.Null,
    'Loaded' : IDL.Null,
    'Loading' : IDL.Null,
    'Pending' : IDL.Null,
  });
  const StagedContentInfo = IDL.Record({
    'stager' : IDL.Principal,
    'status' : StagingStatus,
    'staged_at' : IDL.Nat64,
    'title' : IDL.Text,
    'node_count' : IDL.Nat32,
    'content_hash' : IDL.Text,
    'description' : IDL.Text,
    'proposal_id' : IDL.Opt(IDL.Nat64),
  });
  return IDL.Service({
    'add_allowed_stager' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'delete_staged_content' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_all_content_nodes' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Vec(ContentNode), 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_allowed_stagers' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'get_content_chunk' : IDL.Func(
        [IDL.Text, IDL.Nat32, IDL.Nat32],
        [IDL.Vec(ContentNode)],
        ['query'],
      ),
    'get_governance_canister_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_learning_engine_id' : IDL.Func([], [IDL.Principal], ['query']),
    'get_staged_by_stager' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(StagedContentInfo)],
        ['query'],
      ),
    'get_staged_content_info' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(StagedContentInfo)],
        ['query'],
      ),
    'get_staged_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'list_staged_content' : IDL.Func(
        [],
        [IDL.Vec(StagedContentInfo)],
        ['query'],
      ),
    'mark_loaded' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'mark_loading' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'mark_rejected' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'remove_allowed_stager' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_governance_canister_id' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_learning_engine_id' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'set_proposal_id' : IDL.Func(
        [IDL.Text, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'stage_content' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Vec(ContentNode)],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'staged_content_exists' : IDL.Func([IDL.Text], [IDL.Bool], ['query']),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_engine_id' : IDL.Principal,
    'governance_canister_id' : IDL.Principal,
  });
  return [InitArgs];
};
