import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface FileMetadata {
  'hash' : string,
  'media_type' : MediaType,
  'size' : bigint,
  'content_type' : string,
  'filename' : string,
  'chunk_count' : number,
  'uploader' : Principal,
  'uploaded_at' : bigint,
}
export interface InitArgs { 'allowed_uploaders' : Array<Principal> }
export type MediaType = { 'PDF' : null } |
  { 'Image' : null } |
  { 'Audio' : null } |
  { 'Other' : null } |
  { 'Video' : null };
export interface UploadSession {
  'uploaded_size' : bigint,
  'session_id' : string,
  'chunks_received' : Uint32Array | number[],
  'media_type' : MediaType,
  'content_type' : string,
  'expected_size' : bigint,
  'filename' : string,
  'uploader' : Principal,
  'started_at' : bigint,
}
export interface _SERVICE {
  'add_allowed_uploader' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'cancel_upload' : ActorMethod<[string], { 'Ok' : null } | { 'Err' : string }>,
  'file_exists' : ActorMethod<[string], boolean>,
  'finalize_upload' : ActorMethod<
    [string],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'get_allowed_uploaders' : ActorMethod<[], Array<Principal>>,
  'get_file' : ActorMethod<
    [string],
    { 'Ok' : Uint8Array | number[] } |
      { 'Err' : string }
  >,
  'get_file_chunk' : ActorMethod<
    [string, number],
    [] | [Uint8Array | number[]]
  >,
  'get_file_count' : ActorMethod<[], bigint>,
  'get_file_metadata' : ActorMethod<[string], [] | [FileMetadata]>,
  'get_file_url' : ActorMethod<
    [string],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'get_total_storage' : ActorMethod<[], bigint>,
  'get_upload_session' : ActorMethod<[string], [] | [UploadSession]>,
  'list_files' : ActorMethod<[bigint, bigint], Array<FileMetadata>>,
  'remove_allowed_uploader' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'start_upload' : ActorMethod<
    [string, string, MediaType, bigint],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'upload_chunk' : ActorMethod<
    [string, number, Uint8Array | number[]],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'upload_file' : ActorMethod<
    [string, string, MediaType, Uint8Array | number[]],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
