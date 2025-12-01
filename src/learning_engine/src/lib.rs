use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use std::cell::RefCell;
use std::borrow::Cow;
use candid::{Encode, Decode};

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize)]
struct InitArgs {
    staking_hub_id: Principal,
}

#[derive(CandidType, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UserQuizKey {
    user: Principal,
    unit_id: String,
}

impl Storable for UserQuizKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct QuizQuestion {
    question: String,
    options: Vec<String>,
    answer: u8, // Index of the correct answer
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct LearningUnit {
    unit_id: String,
    unit_title: String,
    chapter_id: String,
    chapter_title: String,
    head_unit_id: String,
    head_unit_title: String,
    content: String,
    paraphrase: String,
    quiz: Vec<QuizQuestion>,
}

impl Storable for LearningUnit {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 50000, // Generous limit for content
        is_fixed_size: false,
    };
}

// Public version without answers
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
    quiz: Vec<PublicQuizQuestion>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct PublicQuizQuestion {
    question: String,
    options: Vec<String>,
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static STAKING_HUB_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    static COMPLETED_QUIZZES: RefCell<StableBTreeMap<UserQuizKey, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    static LEARNING_UNITS: RefCell<StableBTreeMap<String, LearningUnit, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static USER_DAILY_STATS: RefCell<StableBTreeMap<Principal, UserDailyStats, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct UserDailyStats {
    day_index: u64,
    quizzes_taken: u8,
    tokens_earned: u64,
}

impl Storable for UserDailyStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct DailyStatus {
    quizzes_taken: u8,
    daily_limit: u8,
    tokens_earned: u64,
}

#[init]
fn init(args: InitArgs) {
    STAKING_HUB_ID.with(|id| id.borrow_mut().set(args.staking_hub_id).expect("Failed to set Staking Hub ID"));
}

fn get_current_day() -> u64 {
    // Divide nanoseconds by 86,400,000,000,000 to get day index
    ic_cdk::api::time() / 86_400_000_000_000
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

#[update]
async fn submit_quiz(unit_id: String, answers: Vec<u8>) -> Result<u64, String> {
    let user = ic_cdk::caller();
    let key = UserQuizKey { user, unit_id: unit_id.clone() };
    
    // 1. Check Daily Limit
    let current_day = get_current_day();
    let mut daily_stats = USER_DAILY_STATS.with(|s| {
        let s = s.borrow();
        match s.get(&user) {
            Some(stats) if stats.day_index == current_day => stats,
            _ => UserDailyStats { day_index: current_day, quizzes_taken: 0, tokens_earned: 0 }
        }
    });

    if daily_stats.quizzes_taken >= 5 {
        return Err("Daily quiz limit reached (5/5)".to_string());
    }

    // 2. Check if already completed (prevent re-farming same quiz)
    if COMPLETED_QUIZZES.with(|q| q.borrow().contains_key(&key)) {
        return Err("Quiz already completed".to_string());
    }

    // 3. Verify answers & Calculate Score
    let (passed, correct_count, total_questions) = LEARNING_UNITS.with(|u| {
        if let Some(unit) = u.borrow().get(&unit_id) {
            if unit.quiz.len() != answers.len() {
                return (false, 0, 0);
            }
            let mut correct = 0;
            for (i, question) in unit.quiz.iter().enumerate() {
                if question.answer == answers[i] {
                    correct += 1;
                }
            }
            // Pass threshold: 3 out of 5 (or >= 60% generally)
            // If total < 5, we require all correct? Or just use 60% rule?
            // User said "3 out of 5". Let's use generic 60%.
            // Avoid float: (correct * 100) / total >= 60
            let passed = if unit.quiz.len() > 0 {
                (correct * 100) / unit.quiz.len() >= 60
            } else {
                false
            };
            (passed, correct, unit.quiz.len())
        } else {
            (false, 0, 0) // Unit not found
        }
    });

    if total_questions == 0 {
        return Err("Unit not found or empty quiz".to_string());
    }

    // 4. Update Daily Stats (Increment attempts regardless of pass/fail?)
    // "allowed to take 5 quizes". Usually implies attempts.
    daily_stats.quizzes_taken += 1;
    
    if !passed {
        // Save stats (attempt used)
        USER_DAILY_STATS.with(|s| s.borrow_mut().insert(user, daily_stats));
        return Err(format!("Quiz failed. Score: {}/{}. Need 60% to pass.", correct_count, total_questions));
    }

    // 5. Reward (1 Token = 100_000_000 e8s)
    let reward_amount = 100_000_000; 
    daily_stats.tokens_earned += reward_amount;
    
    // Save stats
    USER_DAILY_STATS.with(|s| s.borrow_mut().insert(user, daily_stats));

    // Call Staking Hub to stake rewards (Virtual Staking)
    let staking_hub_id = STAKING_HUB_ID.with(|id| *id.borrow().get());
    
    let _: () = ic_cdk::call(
        staking_hub_id,
        "stake_rewards",
        (user, reward_amount)
    ).await.map_err(|e| format!("Failed to call staking hub: {:?}", e))?;

    // Mark as completed
    COMPLETED_QUIZZES.with(|q| q.borrow_mut().insert(key, true));

    Ok(reward_amount)
}

#[query]
fn get_user_daily_status(user: Principal) -> DailyStatus {
    let current_day = get_current_day();
    USER_DAILY_STATS.with(|s| {
        let s = s.borrow();
        match s.get(&user) {
            Some(stats) if stats.day_index == current_day => DailyStatus {
                quizzes_taken: stats.quizzes_taken,
                daily_limit: 5,
                tokens_earned: stats.tokens_earned,
            },
            _ => DailyStatus {
                quizzes_taken: 0,
                daily_limit: 5,
                tokens_earned: 0,
            }
        }
    })
}

#[query]
fn is_quiz_completed(user: Principal, unit_id: String) -> bool {
    let key = UserQuizKey { user, unit_id };
    COMPLETED_QUIZZES.with(|q| q.borrow().contains_key(&key))
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct LearningUnitMetadata {
    unit_id: String,
    unit_title: String,
    chapter_id: String,
    chapter_title: String,
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
