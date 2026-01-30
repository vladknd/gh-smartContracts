export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'staking_hub_id' : IDL.Principal });
  return IDL.Service({
    'admin_sync_user_subscription' : IDL.Func(
        [IDL.Principal, IDL.Principal, IDL.Bool],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'admin_update_staking_hub' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'confirm_payment' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_staking_hub' : IDL.Func([], [IDL.Principal], ['query']),
    'get_subscription_status' : IDL.Func(
        [IDL.Principal],
        [IDL.Bool],
        ['query'],
      ),
    'request_checkout' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'staking_hub_id' : IDL.Principal });
  return [InitArgs];
};
