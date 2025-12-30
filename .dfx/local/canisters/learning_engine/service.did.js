export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'staking_hub_id' : IDL.Principal });
  const QuizQuestion = IDL.Record({
    'question' : IDL.Text,
    'answer' : IDL.Nat8,
    'options' : IDL.Vec(IDL.Text),
  });
  const LearningUnit = IDL.Record({
    'content' : IDL.Text,
    'head_unit_title' : IDL.Text,
    'paraphrase' : IDL.Text,
    'quiz' : IDL.Vec(QuizQuestion),
    'chapter_title' : IDL.Text,
    'unit_id' : IDL.Text,
    'chapter_id' : IDL.Text,
    'head_unit_id' : IDL.Text,
    'unit_title' : IDL.Text,
  });
  const PublicQuizQuestion = IDL.Record({
    'question' : IDL.Text,
    'options' : IDL.Vec(IDL.Text),
  });
  const PublicLearningUnit = IDL.Record({
    'content' : IDL.Text,
    'head_unit_title' : IDL.Text,
    'paraphrase' : IDL.Text,
    'quiz' : IDL.Vec(PublicQuizQuestion),
    'chapter_title' : IDL.Text,
    'unit_id' : IDL.Text,
    'chapter_id' : IDL.Text,
    'head_unit_id' : IDL.Text,
    'unit_title' : IDL.Text,
  });
  const LearningUnitMetadata = IDL.Record({
    'chapter_title' : IDL.Text,
    'unit_id' : IDL.Text,
    'chapter_id' : IDL.Text,
    'unit_title' : IDL.Text,
  });
  return IDL.Service({
    'add_learning_unit' : IDL.Func(
        [LearningUnit],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_learning_unit' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : PublicLearningUnit, 'Err' : IDL.Text })],
        ['query'],
      ),
    'get_learning_units_metadata' : IDL.Func(
        [],
        [IDL.Vec(LearningUnitMetadata)],
        ['query'],
      ),
    'verify_quiz' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8)],
        [IDL.Bool, IDL.Nat64, IDL.Nat64],
        ['query'],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'staking_hub_id' : IDL.Principal });
  return [InitArgs];
};
