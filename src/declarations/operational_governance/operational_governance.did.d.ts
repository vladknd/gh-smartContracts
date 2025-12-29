import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'ledger_id' : Principal,
  'staking_hub_id' : Principal,
}
/**
 * MMCR status for monitoring
 */
export interface MMCRStatus {
  'last_release_timestamp' : bigint,
  'releases_completed' : bigint,
  'seconds_until_next' : bigint,
  'next_release_amount' : bigint,
  'releases_remaining' : bigint,
}
export interface Proposal {
  'id' : bigint,
  'votes_no' : bigint,
  'recipient' : Principal,
  'description' : string,
  'created_at' : bigint,
  'proposer' : Principal,
  'votes_yes' : bigint,
  'executed' : boolean,
  'amount' : bigint,
}
/**
 * Treasury state tracking balance and allowance
 */
export interface TreasuryState {
  'balance' : bigint,
  /**
   * Spendable amount (increases via MMCR)
   */
  'total_transferred' : bigint,
  'genesis_timestamp' : bigint,
  /**
   * Historical total transferred out
   */
  'mmcr_count' : bigint,
  /**
   * Total MC held (decreases on transfers)
   */
  'allowance' : bigint,
  /**
   * Number of MMCR releases (0-240)
   */
  'last_mmcr_timestamp' : bigint,
}
export interface _SERVICE {
  'create_proposal' : ActorMethod<
    [Principal, bigint, string],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'execute_mmcr' : ActorMethod<[], { 'Ok' : bigint } | { 'Err' : string }>,
  'execute_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_mmcr_status' : ActorMethod<[], MMCRStatus>,
  'get_proposal' : ActorMethod<[bigint], [] | [Proposal]>,
  'get_spendable_balance' : ActorMethod<[], bigint>,
  'get_total_spent' : ActorMethod<[], bigint>,
  'get_treasury_balance' : ActorMethod<[], bigint>,
  'get_treasury_state' : ActorMethod<[], TreasuryState>,
  'vote' : ActorMethod<[bigint, boolean], { 'Ok' : null } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
