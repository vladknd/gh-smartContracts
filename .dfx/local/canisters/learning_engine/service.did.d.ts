import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type ChangeType = { 'Updated' : null } |
  { 'Created' : null } |
  { 'Deleted' : null };
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
export interface ContentSnapshot {
  'modified_at' : bigint,
  'content' : ContentNode,
  'change_type' : ChangeType,
  'modified_by_proposal' : bigint,
}
export interface InitArgs {
  'governance_canister_id' : [] | [Principal],
  'staking_hub_id' : Principal,
}
export interface LoadingJob {
  'last_error' : [] | [string],
  'status' : LoadingStatus,
  'updated_at' : bigint,
  'staging_path' : string,
  'total_units' : number,
  'content_hash' : string,
  'loaded_units' : number,
  'staging_canister' : Principal,
  'proposal_id' : bigint,
  'started_at' : bigint,
}
export type LoadingStatus = { 'Failed' : null } |
  { 'Paused' : null } |
  { 'InProgress' : null } |
  { 'Completed' : null };
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
export interface PublicContentNode {
  'id' : string,
  'media' : [] | [MediaContent],
  'title' : string,
  'updated_at' : bigint,
  'content' : [] | [string],
  'order' : number,
  'paraphrase' : [] | [string],
  'quiz' : [] | [PublicQuizData],
  'description' : [] | [string],
  'created_at' : bigint,
  'display_type' : string,
  'version' : bigint,
  'parent_id' : [] | [string],
}
export interface PublicQuizData { 'questions' : Array<PublicQuizQuestion> }
export interface PublicQuizQuestion {
  'question' : string,
  'options' : Array<string>,
}
export interface QuizCacheData {
  'question_count' : number,
  'content_id' : string,
  'version' : bigint,
  'answer_hashes' : Array<Uint8Array | number[]>,
}
export interface QuizConfig {
  'reward_amount' : bigint,
  'pass_threshold_percent' : number,
  'max_daily_attempts' : number,
}
export interface QuizData { 'questions' : Array<QuizQuestion> }
export interface QuizQuestion {
  'question' : string,
  'answer' : number,
  'options' : Array<string>,
}
export interface _SERVICE {
  'add_content_node' : ActorMethod<
    [ContentNode],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'add_content_nodes' : ActorMethod<
    [Array<ContentNode>],
    { 'Ok' : number } |
      { 'Err' : string }
  >,
  'continue_loading' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'delete_content_node' : ActorMethod<
    [string, bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_all_loading_jobs' : ActorMethod<[], Array<LoadingJob>>,
  'get_changes_by_proposal' : ActorMethod<
    [bigint],
    Array<[string, ChangeType]>
  >,
  'get_children' : ActorMethod<[string], Array<PublicContentNode>>,
  'get_content_at_version' : ActorMethod<[string, bigint], [] | [ContentNode]>,
  'get_content_current_version' : ActorMethod<[string], bigint>,
  'get_content_node' : ActorMethod<[string], [] | [PublicContentNode]>,
  'get_content_stats' : ActorMethod<[], [bigint, bigint]>,
  'get_content_version_global' : ActorMethod<[], bigint>,
  'get_content_version_history' : ActorMethod<
    [string],
    Array<[bigint, ContentSnapshot]>
  >,
  'get_global_quiz_config' : ActorMethod<[], QuizConfig>,
  'get_loading_status' : ActorMethod<[bigint], [] | [LoadingJob]>,
  'get_quiz_data' : ActorMethod<[string], [] | [QuizCacheData]>,
  'get_root_nodes' : ActorMethod<[], Array<PublicContentNode>>,
  'resume_loading' : ActorMethod<
    [bigint],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'start_content_load' : ActorMethod<
    [bigint, Principal, string, string, number],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'update_global_quiz_config' : ActorMethod<
    [[] | [bigint], [] | [number], [] | [number]],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'verify_quiz' : ActorMethod<
    [string, Uint8Array | number[]],
    [boolean, bigint, bigint]
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
