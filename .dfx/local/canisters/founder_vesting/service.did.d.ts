import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'founder1' : Principal,
  'founder2' : Principal,
  'ledger_id' : Principal,
}
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
  'claim_vested' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'get_all_vesting_schedules' : ActorMethod<[], Array<VestingStatus>>,
  'get_genesis_timestamp' : ActorMethod<[], bigint>,
  'get_total_unclaimed' : ActorMethod<[], bigint>,
  'get_vesting_status' : ActorMethod<[Principal], [] | [VestingStatus]>,
  'is_founder' : ActorMethod<[Principal], boolean>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
