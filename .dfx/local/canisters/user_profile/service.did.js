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
    'last_reward_index' : IDL.Nat,
    'gender' : IDL.Text,
  });
  const UserDailyStats = IDL.Record({
    'quizzes_taken' : IDL.Nat8,
    'tokens_earned' : IDL.Nat64,
    'day_index' : IDL.Nat64,
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
    'get_user_daily_status' : IDL.Func(
        [IDL.Principal],
        [UserDailyStats],
        ['query'],
      ),
    'is_quiz_completed' : IDL.Func(
        [IDL.Principal, IDL.Text],
        [IDL.Bool],
        ['query'],
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
    'update_profile' : IDL.Func([UserProfile], [], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'learning_content_id' : IDL.Principal,
    'staking_hub_id' : IDL.Principal,
  });
  return [InitArgs];
};
