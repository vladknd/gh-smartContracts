import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs { 'staking_hub_id' : Principal }
export interface KycStatus {
  'provider' : string,
  'tier' : VerificationTier,
  'user' : Principal,
  'verified_at' : bigint,
}
export type VerificationTier = { 'KYC' : null } |
  { 'None' : null } |
  { 'Human' : null };
export interface _SERVICE {
  'admin_set_staking_hub' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_user_kyc_status' : ActorMethod<[Principal], [] | [KycStatus]>,
  'submit_kyc_data' : ActorMethod<
    [string],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'verify_identity' : ActorMethod<
    [Principal],
    { 'Ok' : VerificationTier } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
