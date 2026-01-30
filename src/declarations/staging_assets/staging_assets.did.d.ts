import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ContentNode {
  'id' : string,
  'media' : [] | [MediaContent],
  'title' : string,
  'updated_at' : bigint,
  'content' : [] | [string],
  'order' : number,
  'paraphrase' : [] | [string],
  'quiz' : [] | [QuizData],
  'description' : [] | [string],
  'created_at' : bigint,
  'display_type' : string,
  'version' : bigint,
  'parent_id' : [] | [string],
}
export interface InitArgs {
  'learning_engine_id' : Principal,
  'governance_canister_id' : Principal,
}
export interface MediaContent {
  'url' : string,
  'duration_seconds' : [] | [number],
  'media_type' : MediaType,
  'file_hash' : [] | [string],
  'thumbnail_url' : [] | [string],
}
export type MediaType = { 'PDF' : null } |
  { 'Image' : null } |
  { 'Audio' : null } |
  { 'Video' : null };
export interface QuizData { 'questions' : Array<QuizQuestion> }
export interface QuizQuestion {
  'question' : string,
  'answer' : number,
  'options' : Array<string>,
}
export interface StagedContentInfo {
  'stager' : Principal,
  'status' : StagingStatus,
  'staged_at' : bigint,
  'title' : string,
  'node_count' : number,
  'content_hash' : string,
  'description' : string,
  'proposal_id' : [] | [bigint],
}
export type StagingStatus = { 'ProposalCreated' : null } |
  { 'Rejected' : null } |
  { 'Loaded' : null } |
  { 'Loading' : null } |
  { 'Pending' : null };
export interface _SERVICE {
  'add_allowed_stager' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'delete_staged_content' : ActorMethod<
    [string],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_all_content_nodes' : ActorMethod<
    [string],
    { 'Ok' : Array<ContentNode> } |
      { 'Err' : string }
  >,
  'get_allowed_stagers' : ActorMethod<[], Array<Principal>>,
  'get_content_chunk' : ActorMethod<
    [string, number, number],
    Array<ContentNode>
  >,
  'get_governance_canister_id' : ActorMethod<[], Principal>,
  'get_learning_engine_id' : ActorMethod<[], Principal>,
  'get_staged_by_stager' : ActorMethod<[Principal], Array<StagedContentInfo>>,
  'get_staged_content_info' : ActorMethod<[string], [] | [StagedContentInfo]>,
  'get_staged_count' : ActorMethod<[], bigint>,
  'list_staged_content' : ActorMethod<[], Array<StagedContentInfo>>,
  'mark_loaded' : ActorMethod<[string], { 'Ok' : null } | { 'Err' : string }>,
  'mark_loading' : ActorMethod<[string], { 'Ok' : null } | { 'Err' : string }>,
  'mark_rejected' : ActorMethod<[string], { 'Ok' : null } | { 'Err' : string }>,
  'remove_allowed_stager' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_governance_canister_id' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_learning_engine_id' : ActorMethod<
    [Principal],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'set_proposal_id' : ActorMethod<
    [string, bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'stage_content' : ActorMethod<
    [string, string, Array<ContentNode>],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'staged_content_exists' : ActorMethod<[string], boolean>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
