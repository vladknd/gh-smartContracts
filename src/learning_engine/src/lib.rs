use std::cell::RefCell;
use std::borrow::Cow;
use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
use candid::{CandidType, Deserialize, Principal};
use candid::{Encode, Decode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ============================================================================
// LEARNING ENGINE CANISTER
// ============================================================================
// 
// This canister stores educational content and quiz questions.
// It is STATELESS with respect to user progress - all user state is in user_profile.
// 
// Key responsibilities:
// - Store learning units (content + quizzes)
// - Verify quiz answers (called by user_profile shards)
// - Return content to frontends (without answers)

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize)]
struct InitArgs {
    /// Principal ID of the staking hub (for future use)
    staking_hub_id: Principal,
}

/// A single quiz question with answer
/// 
/// The answer field contains the correct option index.
/// This is never exposed to users directly.
#[derive(CandidType, Deserialize, Clone, Debug)]
struct QuizQuestion {
    /// The question text
    question: String,
    /// List of answer options
    options: Vec<String>,
    /// Index of the correct answer (0-based)
    answer: u8,
}

/// A learning unit containing educational content and quiz
/// 
/// Learning units are organized hierarchically:
/// - Head Unit (topic) > Chapter > Unit
#[derive(CandidType, Deserialize, Clone, Debug)]
struct LearningUnit {
    /// Unique identifier for this unit
    unit_id: String,
    /// Display title for this unit
    unit_title: String,
    /// Parent chapter ID
    chapter_id: String,
    /// Parent chapter title
    chapter_title: String,
    /// Parent head unit (topic) ID
    head_unit_id: String,
    /// Parent head unit (topic) title
    head_unit_title: String,
    /// Main educational content (markdown supported)
    content: String,
    /// Summary/paraphrase of the content
    paraphrase: String,
    /// Quiz questions (with answers - internal only)
    quiz: Vec<QuizQuestion>,
}

impl Storable for LearningUnit {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode LearningUnit")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50000, // Generous limit for content
        is_fixed_size: false,
    };
}

/// Public version of LearningUnit - EXCLUDES ANSWERS
/// 
/// This is returned to users via get_learning_unit.
/// The quiz options are shown but not the correct answer index.
#[derive(CandidType, Deserialize, Clone, Debug)]
struct PublicLearningUnit {
    unit_id: String,
    unit_title: String,
    chapter_id: String,
    chapter_title: String,
    head_unit_id: String,
    head_unit_title: String,
    content: String,
    paraphrase: String,
    /// Quiz questions WITHOUT answers
    quiz: Vec<PublicQuizQuestion>,
}

/// Public version of QuizQuestion - EXCLUDES ANSWER
#[derive(CandidType, Deserialize, Clone, Debug)]
struct PublicQuizQuestion {
    question: String,
    options: Vec<String>,
    // Note: answer field is intentionally omitted
}

/// Metadata for content discovery/listing
#[derive(CandidType, Deserialize, Clone, Debug)]
struct LearningUnitMetadata {
    unit_id: String,
    unit_title: String,
    chapter_id: String,
    chapter_title: String,
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================
// 
// Memory IDs:
//   0 - STAKING_HUB_ID: Configuration
//   2 - LEARNING_UNITS: Content storage
//   (1, 3 reserved - previously used for user state, now in user_profile)

thread_local! {
    /// Memory manager for allocating virtual memory regions
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    /// Principal ID of the staking hub canister
    /// Stored for potential future use (not currently used)
    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Map of unit_id -> LearningUnit
    /// Contains all educational content and quiz questions
    /// Key is the unit_id string for O(log n) lookups
    static LEARNING_UNITS: RefCell<StableBTreeMap<String, LearningUnit, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );
}

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
}

#[update]
fn add_learning_unit(unit: LearningUnit) -> Result<(), String> {
    // In a production environment, you should check if the caller is an admin.
    LEARNING_UNITS.with(|u| u.borrow_mut().insert(unit.unit_id.clone(), unit));
    Ok(())
}

#[query]
fn get_learning_unit(unit_id: String) -> Result<PublicLearningUnit, String> {
    LEARNING_UNITS.with(|u| {
        if let Some(unit) = u.borrow().get(&unit_id) {
            let public_quiz = unit.quiz.iter().map(|q| PublicQuizQuestion {
                question: q.question.clone(),
                options: q.options.clone(),
            }).collect();

            Ok(PublicLearningUnit {
                unit_id: unit.unit_id,
                unit_title: unit.unit_title,
                chapter_id: unit.chapter_id,
                chapter_title: unit.chapter_title,
                head_unit_id: unit.head_unit_id,
                head_unit_title: unit.head_unit_title,
                content: unit.content,
                paraphrase: unit.paraphrase,
                quiz: public_quiz,
            })
        } else {
            Err("Unit not found".to_string())
        }
    })
}

// New: Stateless Verification Function called by User Profile Canisters
#[query]
fn verify_quiz(unit_id: String, answers: Vec<u8>) -> (bool, u64, u64) {
    LEARNING_UNITS.with(|u| {
        if let Some(unit) = u.borrow().get(&unit_id) {
            if unit.quiz.len() != answers.len() {
                return (false, 0, unit.quiz.len() as u64);
            }
            let mut correct = 0;
            for (i, question) in unit.quiz.iter().enumerate() {
                if question.answer == answers[i] {
                    correct += 1;
                }
            }
            
            // Pass threshold: 60%
            let passed = if unit.quiz.len() > 0 {
                (correct * 100) / unit.quiz.len() >= 60
            } else {
                false
            };
            
            (passed, correct as u64, unit.quiz.len() as u64)
        } else {
            (false, 0, 0) // Unit not found
        }
    })
}

#[query]
fn get_learning_units_metadata() -> Vec<LearningUnitMetadata> {
    LEARNING_UNITS.with(|u| {
        u.borrow().iter().map(|(_, unit)| LearningUnitMetadata {
            unit_id: unit.unit_id.clone(),
            unit_title: unit.unit_title.clone(),
            chapter_id: unit.chapter_id.clone(),
            chapter_title: unit.chapter_title.clone(),
        }).collect()
    })
}

ic_cdk::export_candid!();
