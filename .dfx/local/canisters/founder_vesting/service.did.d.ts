import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs { 'ledger_id' : Principal }
export interface VestingSchedule {
  'claimed' : bigint,
  'vesting_start' : bigint,
  'founder' : Principal,
  'total_allocation' : bigint,
}
export interface VestingStatus {
  'years_elapsed' : bigint,
  'claimed' : bigint,
  'claimable' : bigint,
  'founder' : Principal,
  'vested' : bigint,
  'total_allocation' : bigint,
  'unlock_percentage' : bigint,
}
export interface _SERVICE {
  'admin_claim_vested_at' : ActorMethod<
    [bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'admin_register_founder' : ActorMethod<
    [Principal, bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'admin_set_genesis_timestamp' : ActorMethod<[bigint], undefined>,
  'claim_vested' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'get_all_vesting_schedules' : ActorMethod<[], Array<VestingStatus>>,
  'get_genesis_timestamp' : ActorMethod<[], bigint>,
  'get_total_unclaimed' : ActorMethod<[], bigint>,
  'get_vesting_status' : ActorMethod<[Principal], [] | [VestingStatus]>,
  'is_founder' : ActorMethod<[Principal], boolean>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
