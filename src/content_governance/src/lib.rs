use ic_cdk::init;
use ic_cdk::query;
use candid::candid_method;

#[init]
fn init() {
    // Initialization logic
}

#[query]
#[candid_method(query)]
fn get_book_count() -> u64 {
    0
}

ic_cdk::export_candid!();
