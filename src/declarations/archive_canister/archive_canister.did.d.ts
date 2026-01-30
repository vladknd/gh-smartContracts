import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ArchiveStats {
  'is_full' : boolean,
  'size_bytes' : bigint,
  'parent_shard' : Principal,
  'entry_count' : bigint,
  'next_archive' : [] | [Principal],
}
export interface ArchivedTransaction {
  'transaction_type' : string,
  'metadata' : string,
  'timestamp' : bigint,
  'amount' : bigint,
  'sequence' : bigint,
  'archived_at' : bigint,
}
export interface InitArgs { 'parent_shard_id' : Principal }
export interface TransactionToArchive {
  'transaction_type' : string,
  'metadata' : string,
  'timestamp' : bigint,
  'amount' : bigint,
  'sequence' : bigint,
}
export interface _SERVICE {
  'get_archived_count' : ActorMethod<[Principal], bigint>,
  'get_archived_transactions' : ActorMethod<
    [Principal, [] | [bigint], bigint],
    Array<ArchivedTransaction>
  >,
  'get_parent_shard' : ActorMethod<[], Principal>,
  'get_stats' : ActorMethod<[], ArchiveStats>,
  'get_total_archived_count' : ActorMethod<[], bigint>,
  'receive_archive_batch' : ActorMethod<
    [Principal, Array<TransactionToArchive>],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'set_next_archive' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
