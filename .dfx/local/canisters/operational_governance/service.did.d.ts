import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'ledger_id' : Principal,
  'staking_hub_id' : Principal,
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
export interface _SERVICE {
  'create_proposal' : ActorMethod<
    [Principal, bigint, string],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'execute_proposal' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_proposal' : ActorMethod<[bigint], [] | [Proposal]>,
  'get_total_spent' : ActorMethod<[], bigint>,
  'vote' : ActorMethod<[bigint, boolean], { 'Ok' : null } | { 'Err' : string }>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
