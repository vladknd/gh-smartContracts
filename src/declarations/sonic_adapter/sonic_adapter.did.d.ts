import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface InitArgs {
  'owner' : Principal,
  'ghc_ledger_id' : Principal,
  'usdc_ledger_id' : Principal,
  'sonic_canister_id' : Principal,
}
export interface LaunchIcoArgs { 'usdc_amount' : bigint, 'ghc_amount' : bigint }
export interface _SERVICE {
  'add_liquidity' : ActorMethod<
    [Principal, Principal, bigint, bigint],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'launch_ico' : ActorMethod<
    [LaunchIcoArgs],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'swap' : ActorMethod<
    [Principal, bigint, Principal, bigint],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
