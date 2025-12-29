import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

/**
 * Founder Vesting Canister Interface
 * Manages time-locked founder token allocations with 10%/year vesting
 */
export interface InitArgs {
  /**
   * GHC Ledger canister ID
   */
  'founder1' : Principal,
  /**
   * Founder 1 principal (0.35B allocation)
   */
  'founder2' : Principal,
  'ledger_id' : Principal,
}
/**
 * Internal vesting schedule (not exposed directly)
 */
export interface VestingSchedule {
  'claimed' : bigint,
  'vesting_start' : bigint,
  'founder' : Principal,
  'total_allocation' : bigint,
}
/**
 * Public vesting status for queries
 */
export interface VestingStatus {
  /**
   * Available to claim now (in e8s)
   */
  'years_elapsed' : bigint,
  /**
   * Currently vested/unlocked (in e8s)
   */
  'claimed' : bigint,
  /**
   * Already claimed (in e8s)
   */
  'claimable' : bigint,
  'founder' : Principal,
  /**
   * Total tokens allocated (in e8s)
   */
  'vested' : bigint,
  /**
   * Founder principal ID
   */
  'total_allocation' : bigint,
  /**
   * Years since vesting start
   */
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
