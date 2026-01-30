export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'staking_hub_id' : IDL.Principal });
  const VerificationTier = IDL.Variant({
    'KYC' : IDL.Null,
    'None' : IDL.Null,
    'Human' : IDL.Null,
  });
  const KycStatus = IDL.Record({
    'provider' : IDL.Text,
    'tier' : VerificationTier,
    'user' : IDL.Principal,
    'verified_at' : IDL.Nat64,
  });
  return IDL.Service({
    'admin_set_staking_hub' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text })],
        [],
      ),
    'get_user_kyc_status' : IDL.Func(
        [IDL.Principal],
        [IDL.Opt(KycStatus)],
        ['query'],
      ),
    'submit_kyc_data' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'verify_identity' : IDL.Func(
        [IDL.Principal],
        [IDL.Variant({ 'Ok' : VerificationTier, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({ 'staking_hub_id' : IDL.Principal });
  return [InitArgs];
};
