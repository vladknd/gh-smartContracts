import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface DailyStatus {
  'quizzes_taken' : number,
  'tokens_earned' : bigint,
  'daily_limit' : number,
}
export interface InitArgs { 'staking_hub_id' : Principal }
export interface LearningUnit {
  'content' : string,
  'head_unit_title' : string,
  'paraphrase' : string,
  'quiz' : Array<QuizQuestion>,
  'chapter_title' : string,
  'unit_id' : string,
  'chapter_id' : string,
  'head_unit_id' : string,
  'unit_title' : string,
}
export interface LearningUnitMetadata {
  'chapter_title' : string,
  'unit_id' : string,
  'chapter_id' : string,
  'unit_title' : string,
}
export interface PublicLearningUnit {
  'content' : string,
  'head_unit_title' : string,
  'paraphrase' : string,
  'quiz' : Array<PublicQuizQuestion>,
  'chapter_title' : string,
  'unit_id' : string,
  'chapter_id' : string,
  'head_unit_id' : string,
  'unit_title' : string,
}
export interface PublicQuizQuestion {
  'question' : string,
  'options' : Array<string>,
}
export interface QuizQuestion {
  'question' : string,
  'answer' : number,
  'options' : Array<string>,
}
export interface _SERVICE {
  'add_learning_unit' : ActorMethod<
    [LearningUnit],
    { 'Ok' : null } |
      { 'Err' : string }
  >,
  'get_learning_unit' : ActorMethod<
    [string],
    { 'Ok' : PublicLearningUnit } |
      { 'Err' : string }
  >,
  'get_learning_units_metadata' : ActorMethod<[], Array<LearningUnitMetadata>>,
  'get_user_daily_status' : ActorMethod<[Principal], DailyStatus>,
  'is_quiz_completed' : ActorMethod<[Principal, string], boolean>,
  'submit_quiz' : ActorMethod<
    [string, Uint8Array | number[]],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
