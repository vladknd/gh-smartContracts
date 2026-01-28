export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'governance_canister_id' : IDL.Opt(IDL.Principal),
    'staking_hub_id' : IDL.Principal,
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
  const LoadingStatus = IDL.Variant({
    'Failed' : IDL.Null,
    'Paused' : IDL.Null,
    'InProgress' : IDL.Null,
    'Completed' : IDL.Null,
  });
  const LoadingJob = IDL.Record({
    'last_error' : IDL.Opt(IDL.Text),
    'status' : LoadingStatus,
    'updated_at' : IDL.Nat64,
    'staging_path' : IDL.Text,
    'total_units' : IDL.Nat32,
    'content_hash' : IDL.Text,
    'loaded_units' : IDL.Nat32,
    'staging_canister' : IDL.Principal,
    'proposal_id' : IDL.Nat64,
    'started_at' : IDL.Nat64,
  });
  const QuizCacheData = IDL.Record({
    'question_count' : IDL.Nat8,
    'content_id' : IDL.Text,
    'version' : IDL.Nat64,
    'answer_hashes' : IDL.Vec(IDL.Vec(IDL.Nat8)),
  });
  const ChangeType = IDL.Variant({
    'Updated' : IDL.Null,
    'Created' : IDL.Null,
    'Deleted' : IDL.Null,
  });
  const PublicQuizQuestion = IDL.Record({
    'question' : IDL.Text,
    'options' : IDL.Vec(IDL.Text),
  });
  const PublicQuizData = IDL.Record({
    'questions' : IDL.Vec(PublicQuizQuestion),
  });
  const PublicContentNode = IDL.Record({
    'id' : IDL.Text,
    'media' : IDL.Opt(MediaContent),
    'title' : IDL.Text,
    'updated_at' : IDL.Nat64,
    'content' : IDL.Opt(IDL.Text),
    'order' : IDL.Nat32,
    'paraphrase' : IDL.Opt(IDL.Text),
    'quiz' : IDL.Opt(PublicQuizData),
    'description' : IDL.Opt(IDL.Text),
    'created_at' : IDL.Nat64,
    'display_type' : IDL.Text,
    'version' : IDL.Nat64,
    'parent_id' : IDL.Opt(IDL.Text),
  });
  const ContentSnapshot = IDL.Record({
    'modified_at' : IDL.Nat64,
    'content' : ContentNode,
    'change_type' : ChangeType,
    'modified_by_proposal' : IDL.Nat64,
  });
  return IDL.Service({
    'add_content_node' : IDL.Func(
        [ContentNode],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'add_content_nodes' : IDL.Func(
        [IDL.Vec(ContentNode)],
        [IDL.Variant({ 'Ok' : IDL.Nat32, 'Err' : IDL.Text })],
        [],
      ),
    'continue_loading' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'delete_content_node' : IDL.Func(
        [IDL.Text, IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_all_loading_jobs' : IDL.Func([], [IDL.Vec(LoadingJob)], ['query']),
    'get_all_quiz_cache_data' : IDL.Func(
        [],
        [IDL.Vec(IDL.Tuple(IDL.Text, QuizCacheData))],
        ['query'],
      ),
    'get_changes_by_proposal' : IDL.Func(
        [IDL.Nat64],
        [IDL.Vec(IDL.Tuple(IDL.Text, ChangeType))],
        ['query'],
      ),
    'get_children' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(PublicContentNode)],
        ['query'],
      ),
    'get_content_at_version' : IDL.Func(
        [IDL.Text, IDL.Nat64],
        [IDL.Opt(ContentNode)],
        ['query'],
      ),
    'get_content_current_version' : IDL.Func(
        [IDL.Text],
        [IDL.Nat64],
        ['query'],
      ),
    'get_content_node' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(PublicContentNode)],
        ['query'],
      ),
    'get_content_stats' : IDL.Func([], [IDL.Nat64, IDL.Nat64], ['query']),
    'get_content_version_global' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_content_version_history' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(IDL.Tuple(IDL.Nat64, ContentSnapshot))],
        ['query'],
      ),
    'get_loading_status' : IDL.Func(
        [IDL.Nat64],
        [IDL.Opt(LoadingJob)],
        ['query'],
      ),
    'get_quiz_data' : IDL.Func([IDL.Text], [IDL.Opt(QuizCacheData)], ['query']),
    'get_root_nodes' : IDL.Func([], [IDL.Vec(PublicContentNode)], ['query']),
    'resume_loading' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'start_content_load' : IDL.Func(
        [IDL.Nat64, IDL.Principal, IDL.Text, IDL.Text, IDL.Nat32],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'verify_quiz' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8)],
        [IDL.Bool, IDL.Nat64, IDL.Nat64],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'governance_canister_id' : IDL.Opt(IDL.Principal),
    'staking_hub_id' : IDL.Principal,
  });
  return [InitArgs];
};
