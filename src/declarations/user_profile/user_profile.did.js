export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  const UserProfile = IDL.Record({
    'name' : IDL.Text,
    'education' : IDL.Text,
    'email' : IDL.Text,
    'staked_balance' : IDL.Nat64,
    'gender' : IDL.Text,
    'transaction_count' : IDL.Nat64,
  });
  const UserDailyStats = IDL.Record({
    'quizzes_taken' : IDL.Nat8,
    'tokens_earned' : IDL.Nat64,
    'day_index' : IDL.Nat64,
  });
  const TransactionType = IDL.Variant({
    'Unstake' : IDL.Null,
    'QuizReward' : IDL.Null,
  });
  const TransactionRecord = IDL.Record({
    'timestamp' : IDL.Nat64,
    'tx_type' : TransactionType,
    'amount' : IDL.Nat64,
  });
  const UserProfileUpdate = IDL.Record({
    'name' : IDL.Text,
    'education' : IDL.Text,
    'email' : IDL.Text,
    'gender' : IDL.Text,
  });
  return IDL.Service({
    'debug_force_sync' : IDL.Func(
        [],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_profile' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(UserProfile)],
        ['query'],
      ),
    'get_user_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_user_daily_status' : IDL.Func(
        [IDL.Principal],
        [UserDailyStats],
        ['query'],
      ),
    'get_user_transactions' : IDL.Func(
        [IDL.Principal],
        [IDL.Vec(TransactionRecord)],
        ['query'],
      ),
    'is_quiz_completed' : IDL.Func(
        [IDL.Principal, IDL.Text],
        [IDL.Bool],
        ['query'],
      ),
    'register_user' : IDL.Func(
        [UserProfileUpdate],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'submit_quiz' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8)],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'unstake' : IDL.Func(
        [IDL.Nat64],
        [IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text })],
        [],
      ),
    'update_profile' : IDL.Func(
        [UserProfileUpdate],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  return [InitArgs];
};
