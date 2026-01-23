import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

/**
 * Eastern Time components: (year, month, day, hour, minute, second, is_dst)
 */
export interface EasternTimeInfo {
  'day' : number,
  'month' : number,
  'hour' : number,
  'year' : number,
  'minute' : number,
  'second' : number,
  'is_dst' : boolean,
}
export interface ExecuteTransferInput {
  'recipient' : Principal,
  'proposal_id' : bigint,
  'amount' : bigint,
  'token_type' : TokenType,
}
export interface InitArgs {
  'ledger_id' : Principal,
  'governance_canister_id' : Principal,
}
/**
 * MMCR execution check result: (can_execute, message)
 */
export interface MMCRExecutionCheck {
  'can_execute' : boolean,
  'message' : string,
}
export interface MMCRStatus {
  'last_release_timestamp' : bigint,
  'releases_completed' : bigint,
  'next_scheduled_year' : number,
  /**
   * Next scheduled MMCR date
   */
  'next_scheduled_month' : number,
  'seconds_until_next' : bigint,
  'next_release_amount' : bigint,
  'releases_remaining' : bigint,
}
/**
 * ============================================================================
 * Treasury Canister Candid Interface
 * ============================================================================
 * Token custody, balance tracking, and transfer execution
 * Created: January 2026
 * Updated: January 2026 - Added calendar-based MMCR scheduling (1st of month at 12am ET)
 */
export type TokenType = { 'GHC' : null } |
  { 'ICP' : null } |
  { 'USDC' : null };
export interface TreasuryState {
  'balance' : bigint,
  'total_transferred' : bigint,
  'genesis_timestamp' : bigint,
  'mmcr_count' : bigint,
  'last_mmcr_year' : number,
  'allowance' : bigint,
  'last_mmcr_timestamp' : bigint,
  /**
   * Calendar-based tracking for MMCR scheduling
   */
  'last_mmcr_month' : number,
}
export interface _SERVICE {
  'can_execute_mmcr_now' : ActorMethod<[], [boolean, string]>,
  'can_transfer' : ActorMethod<[bigint, TokenType], boolean>,
  'execute_mmcr' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'execute_transfer' : ActorMethod<
    [ExecuteTransferInput],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'force_execute_mmcr' : ActorMethod<
    [],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'get_current_eastern_time' : ActorMethod<
    [],
    [number, number, number, number, number, number, boolean]
  >,
  'get_governance_canister_id' : ActorMethod<[], Principal>,
  'get_ledger_id' : ActorMethod<[], Principal>,
  'get_mmcr_status' : ActorMethod<[], MMCRStatus>,
  'get_spendable_balance' : ActorMethod<[], bigint>,
  'get_test_timestamps_for_year' : ActorMethod<
    [number],
    Array<[number, bigint, boolean]>
  >,
  'get_treasury_balance' : ActorMethod<[], bigint>,
  'get_treasury_state' : ActorMethod<[], TreasuryState>,
  'reset_mmcr_for_testing' : ActorMethod<
    [],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'set_governance_canister_id' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'simulate_mmcr_at_time' : ActorMethod<
    [bigint],
    [boolean, string, number, number]
  >,
  'test_date_parsing' : ActorMethod<
    [bigint],
    [number, number, number, number, number, number, boolean, number]
  >,
  'test_dst_boundaries' : ActorMethod<
    [number],
    [number, number, number, number]
  >,
  'test_mmcr_window' : ActorMethod<
    [number, number, number, number, number],
    [boolean, bigint, string]
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
