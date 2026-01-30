import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface IcoState {
  'price_per_token_e6' : bigint,
  'total_sold_ghc' : bigint,
  'ghc_ledger_id' : Principal,
  'ghc_decimals' : number,
  'total_raised_usdc' : bigint,
  'ckusdc_ledger_id' : Principal,
  'admin_principal' : Principal,
}
export interface InitArgs {
  'price_per_token_e6' : bigint,
  'ghc_ledger_id' : Principal,
  'ghc_decimals' : number,
  'treasury_principal' : Principal,
  'ckusdc_ledger_id' : Principal,
  'admin_principal' : Principal,
}
export interface _SERVICE {
  'buy_ghc' : ActorMethod<[bigint], { 'Ok' : string } | { 'Err' : string }>,
  'end_sale' : ActorMethod<[], { 'Ok' : string } | { 'Err' : string }>,
  'get_ico_stats' : ActorMethod<[], IcoState>,
  'withdraw_ghc' : ActorMethod<
    [Principal, bigint],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'withdraw_usdc' : ActorMethod<
    [Principal, bigint],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
