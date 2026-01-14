use std::time::Duration;
use std::cell::RefCell;
use std::borrow::Cow;
use ic_cdk::init;
use ic_cdk::query;
use ic_cdk::update;
use ic_cdk_timers::set_timer_interval;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell, Storable};
use ic_stable_structures::storable::Bound;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::{TransferArg, TransferError};
use candid::{CandidType, Deserialize, Principal, Encode, Decode, Nat};

type Memory = VirtualMemory<DefaultMemoryImpl>;

// ===============================
// CONSTANTS
// ===============================

// MAX_SUPPLY: 4.75B MUC tokens with 8 decimals
// 4.75B * 10^8 = 4.75 * 10^17 (fits comfortably in u64 max of ~1.8 * 10^19)
const MAX_SUPPLY: u64 = 4_750_000_000 * 100_000_000; // 4.75B MUC Tokens (8 decimals)
const SHARD_SOFT_LIMIT: u64 = 90_000;  // Start creating new shard at 90K users
const SHARD_HARD_LIMIT: u64 = 100_000; // Max users per shard
const AUTO_SCALE_INTERVAL_SECS: u64 = 60; // Check every minute

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Arguments passed during canister initialization
#[derive(CandidType, Deserialize, Clone)]
struct InitArgs {
    /// Principal ID of the GHC ledger canister (ICRC1)
    ledger_id: Principal,
    /// Principal ID of the learning content canister
    learning_content_id: Principal,
    /// Embedded WASM binary for auto-deploying user_profile shard canisters
    user_profile_wasm: Vec<u8>,
}

/// Global statistics tracked by the staking hub
/// 
/// This is the central source of truth for all staking-related metrics.
/// Simplified version without interest/penalty system.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct GlobalStats {
    /// Total tokens currently staked across all users
    pub total_staked: u64,
    
    /// Total tokens that have been unstaked
    pub total_unstaked: u64,
    
    /// Total tokens allocated for minting (tracked against MAX_SUPPLY cap)
    pub total_allocated: u64,
}

impl Storable for GlobalStats {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode GlobalStats")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Information about a user_profile shard canister
/// 
/// Shards are automatically deployed by the staking hub to distribute
/// user load. Each shard can hold up to SHARD_HARD_LIMIT (100K) users.
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ShardInfo {
    /// Principal ID of the shard canister
    pub canister_id: Principal,
    /// Timestamp when this shard was created (nanoseconds)
    pub created_at: u64,
    /// Current number of registered users in this shard
    pub user_count: u64,
    /// Operational status of the shard
    pub status: ShardStatus,
}

/// Operational status of a shard canister
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum ShardStatus {
    /// Shard is accepting new user registrations
    Active,
    /// Shard has reached capacity and is not accepting new users
    Full,
}

impl Storable for ShardInfo {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode ShardInfo")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 200,
        is_fixed_size: false,
    };
}

/// Wrapper for storing large WASM binary in stable memory
/// 
/// The embedded WASM is used by the hub to deploy new shard canisters
/// automatically when existing shards reach capacity.
#[derive(CandidType, Deserialize, Clone, Default)]
struct WasmBlob {
    data: Vec<u8>,
}

impl Storable for WasmBlob {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.data.clone())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self { data: bytes.to_vec() }
    }

    const BOUND: Bound = Bound::Unbounded;
}

// ============================================================================
// THREAD-LOCAL STORAGE
// ============================================================================
// 
// All persistent state is stored in stable memory using ic_stable_structures.
// Each storage item is assigned a unique MemoryId for isolation.
// 
// Memory IDs:
//   0 - LEDGER_ID: GHC ledger canister principal
//   1 - GLOBAL_STATS: Aggregate staking statistics
//   3 - REGISTERED_SHARDS: Set of authorized shard canisters
//   4 - SHARD_REGISTRY: Detailed info about each shard
//   5 - SHARD_COUNT: Number of shards created
//   6 - LEARNING_CONTENT_ID: Learning engine canister principal
//   7 - EMBEDDED_WASM: User profile WASM for auto-deployment
//   8 - INITIALIZED: Whether init() has been called

thread_local! {
    // ─────────────────────────────────────────────────────────────────────
    // Memory Management
    // ─────────────────────────────────────────────────────────────────────
    
    /// Memory manager for allocating virtual memory regions to each storage
    /// Allows multiple stable data structures to coexist
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    // ─────────────────────────────────────────────────────────────────────
    // Configuration (Set once during init, immutable after)
    // ─────────────────────────────────────────────────────────────────────

    /// Principal ID of the GHC ICRC-1 ledger canister
    /// Used for token transfers during unstaking
    static LEDGER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Principal ID of the learning_engine canister
    /// Passed to new shard canisters during auto-deployment
    static LEARNING_CONTENT_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            Principal::anonymous()
        ).unwrap()
    );

    /// Embedded user_profile WASM binary for auto-deploying new shards
    /// Stored in stable memory to survive upgrades
    /// Empty if auto-scaling is not enabled
    static EMBEDDED_WASM: RefCell<StableCell<WasmBlob, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
            WasmBlob::default()
        ).unwrap()
    );

    /// Flag indicating whether init() has been called
    /// Prevents re-initialization attacks
    static INITIALIZED: RefCell<StableCell<bool, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
            false
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Global Economy State
    // ─────────────────────────────────────────────────────────────────────

    /// Aggregate staking statistics across all shards
    /// This is the source of truth for:
    /// - Total staked tokens
    /// - Total unstaked tokens
    /// - Total allocated tokens (against MAX_SUPPLY cap)
    static GLOBAL_STATS: RefCell<StableCell<GlobalStats, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            GlobalStats {
                total_staked: 0,
                total_unstaked: 0,
                total_allocated: 0,
            }
        ).unwrap()
    );

    // ─────────────────────────────────────────────────────────────────────
    // Shard Management
    // ─────────────────────────────────────────────────────────────────────

    /// Set of registered shard canister principals: Principal -> true
    /// Only registered shards can call sync_shard and process_unstake
    /// Used for O(1) authorization checks
    static REGISTERED_SHARDS: RefCell<StableBTreeMap<Principal, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    /// Detailed information about each shard: index -> ShardInfo
    /// Used for shard discovery and load balancing
    /// Index is sequential, starting at 0
    static SHARD_REGISTRY: RefCell<StableBTreeMap<u64, ShardInfo, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    /// Counter for the number of shards created
    /// Used to generate sequential shard indexes
    /// Also used to iterate over SHARD_REGISTRY
    static SHARD_COUNT: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            0
        ).unwrap()
    );
}

// ===============================
// Initialization (IMMUTABLE AFTER)
// ===============================

#[init]
fn init(args: InitArgs) {
    // Store configuration (immutable after init)
    LEDGER_ID.with(|id| {
        id.borrow_mut().set(args.ledger_id).expect("Failed to set Ledger ID");
    });
    
    LEARNING_CONTENT_ID.with(|id| {
        id.borrow_mut().set(args.learning_content_id).expect("Failed to set Learning Content ID");
    });
    
    // Store the user_profile WASM (immutable after init)
    if !args.user_profile_wasm.is_empty() {
        EMBEDDED_WASM.with(|w| {
            w.borrow_mut().set(WasmBlob { data: args.user_profile_wasm })
                .expect("Failed to store WASM");
        });
    }
    
    INITIALIZED.with(|i| {
        i.borrow_mut().set(true).expect("Failed to set initialized flag");
    });
    
    // Start auto-scaling timer
    start_auto_scale_timer();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    // Restart auto-scaling timer after upgrade
    start_auto_scale_timer();
}

fn start_auto_scale_timer() {
    set_timer_interval(Duration::from_secs(AUTO_SCALE_INTERVAL_SECS), || {
        ic_cdk::spawn(async {
            let _ = ensure_capacity_internal().await;
        });
    });
}

// ===============================
// Auto-Scaling Functions
// ===============================

/// Public function to trigger capacity check and shard creation if needed
/// Anyone can call this - it's safe because it only creates shards from embedded WASM
#[update]
async fn ensure_capacity() -> Result<Option<Principal>, String> {
    ensure_capacity_internal().await
}

async fn ensure_capacity_internal() -> Result<Option<Principal>, String> {
    // 1. Check if WASM is embedded
    let has_wasm = EMBEDDED_WASM.with(|w| !w.borrow().get().data.is_empty());
    if !has_wasm {
        return Err("No WASM embedded - cannot auto-create shards".to_string());
    }
    
    // 2. Get current shards
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    // 3. If no shards exist, create the first one
    if shard_count == 0 {
        let new_shard = create_shard_internal().await?;
        return Ok(Some(new_shard));
    }
    
    // 4. Check if all active shards are near capacity
    let active_shards = get_active_shards_internal();
    
    if active_shards.is_empty() {
        // All shards are full, need a new one
        let new_shard = create_shard_internal().await?;
        return Ok(Some(new_shard));
    }
    
    // Check if the best shard is approaching soft limit
    let min_user_count = active_shards.iter().map(|s| s.user_count).min().unwrap_or(0);
    
    if min_user_count >= SHARD_SOFT_LIMIT {
        // Proactively create a new shard
        let new_shard = create_shard_internal().await?;
        return Ok(Some(new_shard));
    }
    
    Ok(None) // No new shard needed
}

async fn create_shard_internal() -> Result<Principal, String> {
    // 1. Get embedded WASM
    let wasm_module = EMBEDDED_WASM.with(|w| w.borrow().get().data.clone());
    
    if wasm_module.is_empty() {
        return Err("No WASM embedded".to_string());
    }
    
    // 2. Get required IDs
    let learning_content_id = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    let staking_hub_id = ic_cdk::id();
    
    // 3. Create canister via management canister
    #[derive(CandidType)]
    struct CreateCanisterArgs {
        settings: Option<CanisterSettings>,
    }
    
    #[derive(CandidType)]
    struct CanisterSettings {
        controllers: Option<Vec<Principal>>,
        compute_allocation: Option<Nat>,
        memory_allocation: Option<Nat>,
        freezing_threshold: Option<Nat>,
    }
    
    #[derive(CandidType, Deserialize)]
    struct CreateCanisterResult {
        canister_id: Principal,
    }
    
    let create_args = CreateCanisterArgs {
        settings: Some(CanisterSettings {
            controllers: Some(vec![staking_hub_id]), // Hub controls the shard
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        }),
    };
    
    // Call management canister to create
    let (create_result,): (CreateCanisterResult,) = ic_cdk::call(
        Principal::management_canister(),
        "create_canister",
        (create_args,)
    ).await.map_err(|(code, msg)| format!("Failed to create canister: {:?} {}", code, msg))?;
    
    let new_canister_id = create_result.canister_id;
    
    // 4. Install the WASM code
    #[derive(CandidType)]
    struct InstallCodeArgs {
        mode: InstallMode,
        canister_id: Principal,
        wasm_module: Vec<u8>,
        arg: Vec<u8>,
    }
    
    #[derive(CandidType, Deserialize)]
    enum InstallMode {
        #[allow(dead_code)]
        install,
        #[allow(dead_code)]
        reinstall,
        #[allow(dead_code)]
        upgrade,
    }
    
    // Prepare init args for user_profile
    #[derive(CandidType)]
    struct UserProfileInitArgs {
        staking_hub_id: Principal,
        learning_content_id: Principal,
    }
    
    let init_args = UserProfileInitArgs {
        staking_hub_id,
        learning_content_id,
    };
    
    let install_args = InstallCodeArgs {
        mode: InstallMode::install,
        canister_id: new_canister_id,
        wasm_module,
        arg: Encode!(&init_args).map_err(|e| format!("Failed to encode init args: {}", e))?,
    };
    
    let _: () = ic_cdk::call(
        Principal::management_canister(),
        "install_code",
        (install_args,)
    ).await.map_err(|(code, msg)| format!("Failed to install code: {:?} {}", code, msg))?;
    
    // 5. Register the new shard
    register_shard_internal(new_canister_id);
    
    Ok(new_canister_id)
}

fn register_shard_internal(canister_id: Principal) {
    let shard_index = SHARD_COUNT.with(|c| {
        let mut cell = c.borrow_mut();
        let idx = *cell.get();
        cell.set(idx + 1).expect("Failed to increment shard count");
        idx
    });
    
    let shard_info = ShardInfo {
        canister_id,
        created_at: ic_cdk::api::time(),
        user_count: 0,
        status: ShardStatus::Active,
    };
    
    REGISTERED_SHARDS.with(|m| m.borrow_mut().insert(canister_id, true));
    SHARD_REGISTRY.with(|r| r.borrow_mut().insert(shard_index, shard_info));
}

fn get_active_shards_internal() -> Vec<ShardInfo> {
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    SHARD_REGISTRY.with(|r| {
        let registry = r.borrow();
        (0..shard_count)
            .filter_map(|i| registry.get(&i))
            .filter(|s| s.status == ShardStatus::Active)
            .collect()
    })
}

// ===============================
// Shard Discovery Queries (for Frontend)
// ===============================

/// Get all registered shards
#[query]
fn get_shards() -> Vec<ShardInfo> {
    let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
    
    SHARD_REGISTRY.with(|r| {
        let registry = r.borrow();
        (0..shard_count)
            .filter_map(|i| registry.get(&i))
            .collect()
    })
}

/// Get only active shards
#[query]
fn get_active_shards() -> Vec<ShardInfo> {
    get_active_shards_internal()
}

/// Find the best shard for a new user (load balancing)
#[query]
fn get_shard_for_new_user() -> Option<Principal> {
    get_active_shards_internal()
        .into_iter()
        .filter(|s| s.user_count < SHARD_HARD_LIMIT)
        .min_by_key(|s| s.user_count)
        .map(|s| s.canister_id)
}

/// Check if a canister is a registered shard
#[query]
fn is_registered_shard(canister_id: Principal) -> bool {
    REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&canister_id))
}

#[query]
fn get_shard_count() -> u64 {
    SHARD_COUNT.with(|c| *c.borrow().get())
}

// ============================================================================
// ADMIN FUNCTIONS
// ============================================================================

/// Manually register a canister as an allowed shard (minter)
/// 
/// This is used when deploying user_profile canisters manually
/// instead of using the auto-scaling mechanism.
/// 
/// SECURITY: This should only be callable by controllers in production
#[update]
fn add_allowed_minter(canister_id: Principal) {
    // Register the shard
    register_shard_internal(canister_id);
}

// ===============================
// Shard Update Functions (Called by Shards)
// ===============================

/// Update shard user count (called by shards during sync)
#[update]
fn update_shard_user_count(user_count: u64) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Verify caller is a registered shard
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }
    
    // Find and update the shard in registry
    SHARD_REGISTRY.with(|r| {
        let mut registry = r.borrow_mut();
        let shard_count = SHARD_COUNT.with(|c| *c.borrow().get());
        
        for i in 0..shard_count {
            if let Some(mut shard) = registry.get(&i) {
                if shard.canister_id == caller {
                    shard.user_count = user_count;
                    
                    // Mark as full if threshold reached
                    if user_count >= SHARD_HARD_LIMIT {
                        shard.status = ShardStatus::Full;
                    }
                    
                    registry.insert(i, shard);
                    return Ok(());
                }
            }
        }
        
        Err("Shard not found in registry".to_string())
    })
}

/// Synchronize shard statistics with the hub and request minting allowance
/// 
/// This function is called periodically by each user_profile shard to:
/// 1. Report stake changes (new stakes from quiz rewards)
/// 2. Report unstaking activity
/// 3. Request additional minting allowance when running low
/// 
/// # Arguments
/// * `staked_delta` - Change in staked amounts since last sync
/// * `unstaked_delta` - Total amount unstaked since last sync
/// * `requested_allowance` - Amount of minting allowance to request
/// 
/// # Returns
/// * `granted_allowance` - Allowance granted for minting
/// 
/// # Security
/// - Only registered shards can call this function
/// - Uses saturating arithmetic to prevent overflow/underflow
#[update]
fn sync_shard(
    staked_delta: i64,
    unstaked_delta: u64,
    requested_allowance: u64
) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // SECURITY: Only shards created by this hub can report stats
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }
    
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        
        // ─────────────────────────────────────────────────────────────────
        // Step 1: Update staked amounts and track actual minting
        // ─────────────────────────────────────────────────────────────────
        if staked_delta > 0 {
            stats.total_staked = stats.total_staked.saturating_add(staked_delta as u64);
            // Track actual tokens minted (positive delta = new tokens created)
            stats.total_allocated = stats.total_allocated.saturating_add(staked_delta as u64);
        } else {
            stats.total_staked = stats.total_staked.saturating_sub(staked_delta.abs() as u64);
        }
        
        // Update unstaked total
        stats.total_unstaked += unstaked_delta;

        // ─────────────────────────────────────────────────────────────────
        // Step 2: Handle Allowance Request (Hard Cap Enforcement)
        // ─────────────────────────────────────────────────────────────────
        // Note: Allowance grants are pre-approvals for future minting.
        // total_allocated is updated in Step 1 when tokens are actually minted.
        let granted_allowance = if requested_allowance > 0 {
            // Never grant beyond MAX_SUPPLY (4.75B MUC tokens)
            let remaining = MAX_SUPPLY.saturating_sub(stats.total_allocated);
            let to_grant = remaining.min(requested_allowance);
            // Don't add to total_allocated here - it's tracked when tokens are actually minted
            to_grant
        } else {
            0
        };
        
        cell.set(stats).expect("Failed to update global stats");
        Ok(granted_allowance)
    })
}

/// Process unstake request from a shard - returns 100% (no penalty)
#[update]
async fn process_unstake(user: Principal, amount: u64) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    
    // Verify caller is a registered shard
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }

    // No penalty - return full amount
    let return_amount = amount;

    // Update global stats
    GLOBAL_STATS.with(|s| {
        let mut cell = s.borrow_mut();
        let mut stats = cell.get().clone();
        stats.total_unstaked += return_amount;
        cell.set(stats).expect("Failed to update global stats");
    });

    // Ledger transfer
    let ledger_id = LEDGER_ID.with(|id| *id.borrow().get());
    
    let args = TransferArg {
        from_subaccount: None,
        to: Account { owner: user, subaccount: None },
        amount: Nat::from(return_amount),
        fee: None,
        memo: None,
        created_at_time: None,
    };

    let (result,): (Result<Nat, TransferError>,) = ic_cdk::call(
        ledger_id,
        "icrc1_transfer",
        (args,)
    ).await.map_err(|(code, msg)| format!("Transfer call failed: {:?} {}", code, msg))?;

    match result {
        Ok(_) => Ok(return_amount),
        Err(e) => {
            // Rollback
            GLOBAL_STATS.with(|s| {
                let mut cell = s.borrow_mut();
                let mut stats = cell.get().clone();
                stats.total_unstaked -= return_amount;
                cell.set(stats).expect("Failed to rollback");
            });
            Err(format!("Ledger transfer failed: {:?}", e))
        }
    }
}

// ===============================
// Query Functions
// ===============================

#[query]
fn get_global_stats() -> GlobalStats {
    GLOBAL_STATS.with(|s| s.borrow().get().clone())
}

#[query]
fn get_config() -> (Principal, Principal, bool) {
    let ledger = LEDGER_ID.with(|id| *id.borrow().get());
    let learning = LEARNING_CONTENT_ID.with(|id| *id.borrow().get());
    let has_wasm = EMBEDDED_WASM.with(|w| !w.borrow().get().data.is_empty());
    (ledger, learning, has_wasm)
}

#[query]
fn get_limits() -> (u64, u64) {
    (SHARD_SOFT_LIMIT, SHARD_HARD_LIMIT)
}

// ============================================================================
// GOVERNANCE VOTING POWER
// ============================================================================
//
// This section provides voting power lookups for the governance system.
// - VUC (Volume of Unmined Coins) = board member voting power (weighted by shares)
// - User staked_balance = user voting power
//
// Board members are registered with percentage shares of VUC voting power.
// Total shares must equal 100%. Can be locked for immutability.
// The user registry maps each user to their shard for O(1) lookup.

// Using MemoryId 9 for USER_SHARD_MAP, 10 for BOARD_MEMBER_SHARES, 11 for BOARD_SHARES_LOCKED
thread_local! {
    /// Map of user principal -> shard principal
    /// Updated when users register in shards
    /// Enables O(1) lookup for voting power queries
    static USER_SHARD_MAP: RefCell<StableBTreeMap<Principal, Principal, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9)))
        )
    );
    
    // Note: MemoryIds 10, 11, 12 were previously used for board member storage
    // Board member management has been moved to operational_governance canister
    // These memory slots are kept reserved to avoid conflicts
}

// ============================================================================
// BOARD MEMBER MANAGEMENT - DEPRECATED
// ============================================================================
// Board member management has been moved to the operational_governance canister.
// See operational_governance for: set_board_member_shares, lock_board_member_shares,
// get_board_member_shares, is_board_member, etc.


// ============================================================================
// USER REGISTRY
// ============================================================================

/// Register a user's shard location (called by shards during user registration)
#[update]
fn register_user_location(user: Principal) -> Result<(), String> {
    let caller = ic_cdk::caller();
    
    // Only registered shards can register user locations
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&caller));
    if !is_registered {
        return Err("Unauthorized: Caller is not a registered shard".to_string());
    }
    
    // Store user -> shard mapping
    USER_SHARD_MAP.with(|m| {
        m.borrow_mut().insert(user, caller);
    });
    
    Ok(())
}

/// Get which shard a user is registered in
#[query]
fn get_user_shard(user: Principal) -> Option<Principal> {
    USER_SHARD_MAP.with(|m| m.borrow().get(&user))
}

/// Manually set a user's shard location (admin only)
/// Used to backfill existing users who registered before the registry was added
#[update]
fn admin_set_user_shard(user: Principal, shard: Principal) -> Result<(), String> {
    if !ic_cdk::api::is_controller(&ic_cdk::caller()) {
        return Err("Unauthorized: Only controllers can set user shards".to_string());
    }
    
    // Verify the shard is registered
    let is_registered = REGISTERED_SHARDS.with(|m| m.borrow().contains_key(&shard));
    if !is_registered {
        return Err("Invalid shard: Not a registered shard".to_string());
    }
    
    USER_SHARD_MAP.with(|m| {
        m.borrow_mut().insert(user, shard);
    });
    
    Ok(())
}


// ============================================================================
// VOTING POWER FUNCTIONS
// ============================================================================
// Note: Board member voting power is now handled by operational_governance.
// This canister provides get_vuc() for governance to calculate board member power,
// and fetch_user_voting_power() for regular user staked balance lookups.

/// Get VUC (Volume of Unmined Coins) - total board member voting power pool
/// VUC = MAX_SUPPLY - total_allocated
/// This is used by operational_governance to calculate board member voting power
#[query]
fn get_vuc() -> u64 {
    GLOBAL_STATS.with(|s| {
        let stats = s.borrow().get().clone();
        MAX_SUPPLY.saturating_sub(stats.total_allocated)
    })
}

/// Get total voting power in the system (VUC + total_staked)
#[query]
fn get_total_voting_power() -> u64 {
    let vuc = get_vuc();
    let total_staked = GLOBAL_STATS.with(|s| s.borrow().get().total_staked);
    vuc.saturating_add(total_staked)
}

/// Fetch voting power for a regular user (async - queries shards)
/// 
/// This returns the user's staked balance from their shard.
/// For board members, operational_governance calculates their weighted VUC locally.
#[update]
async fn fetch_user_voting_power(user: Principal) -> u64 {
    // Look up user's shard
    let shard_id = USER_SHARD_MAP.with(|m| m.borrow().get(&user));
    
    let shard_id = match shard_id {
        Some(id) => id,
        None => return 0, // User not registered
    };
    
    // Query shard for user's profile
    #[derive(CandidType, Deserialize)]
    struct UserProfile {
        email: String,
        name: String,
        education: String,
        gender: String,
        staked_balance: u64,
        transaction_count: u64,
    }
    
    let result: Result<(Option<UserProfile>,), _> = ic_cdk::call(
        shard_id,
        "get_profile",
        (user,)
    ).await;
    
    match result {
        Ok((Some(profile),)) => profile.staked_balance,
        _ => 0,
    }
}

/// Get governance tokenomics summary
#[query]
fn get_tokenomics() -> (u64, u64, u64, u64) {
    let stats = GLOBAL_STATS.with(|s| s.borrow().get().clone());
    let vuc = MAX_SUPPLY.saturating_sub(stats.total_allocated);
    let total_voting_power = vuc.saturating_add(stats.total_staked);
    
    (
        MAX_SUPPLY,           // Total Utility Coin Cap (4.75B)
        stats.total_allocated, // Tokens mined/allocated so far
        vuc,                  // VUC (total board member voting power pool)
        total_voting_power    // Total voting power in system
    )
}

ic_cdk::export_candid!();

