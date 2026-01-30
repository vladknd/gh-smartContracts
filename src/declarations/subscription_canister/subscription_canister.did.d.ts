import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs { 'staking_hub_id' : Principal }
export interface SubscriptionRecord {
  'status' : string,
  'session_id' : string,
  'shard_id' : Principal,
  'user' : Principal,
  'timestamp' : bigint,
  'amount' : bigint,
}
export interface _SERVICE {
  'admin_sync_user_subscription' : ActorMethod<
    [Principal, Principal, boolean],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'admin_update_staking_hub' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'confirm_payment' : ActorMethod<
    [string],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_staking_hub' : ActorMethod<[], Principal>,
  'get_subscription_status' : ActorMethod<[Principal], boolean>,
  'request_checkout' : ActorMethod<
    [Principal],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
