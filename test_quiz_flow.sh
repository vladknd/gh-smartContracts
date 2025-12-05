#!/bin/bash

# Set up environment
dfx stop
dfx start --background --clean

# Deploy canisters
echo "Deploying canisters..."
# Use management canister as dummy ledger_id
dfx deploy staking_hub --argument '(record { ledger_id = principal "aaaaa-aa" })'
STAKING_HUB_ID=$(dfx canister id staking_hub)

dfx deploy learning_engine --argument "(record { staking_hub_id = principal \"$STAKING_HUB_ID\" })"

# Add a learning unit
echo "Adding learning unit..."
dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "1.0";
    unit_title = "Test Unit";
    chapter_id = "1";
    chapter_title = "Test Chapter";
    head_unit_id = "1";
    head_unit_title = "Test Head Unit";
    content = "This is test content.";
    paraphrase = "This is test paraphrase.";
    quiz = vec {
        record {
            question = "What is 1+1?";
            options = vec { "1"; "2"; "3" };
            answer = 1;
        };
        record {
            question = "What is 2+2?";
            options = vec { "3"; "4"; "5" };
            answer = 1;
        };
    };
})'

# Get learning unit (verify answers are hidden)
echo "Getting learning unit..."
RESULT=$(dfx canister call learning_engine get_learning_unit '("1.0")')
echo "Result: $RESULT"

if [[ $RESULT == *"answer"* ]]; then
    echo "FAIL: Public unit contains answers!"
    exit 1
else
    echo "PASS: Public unit does not contain answers."
fi

# Submit incorrect quiz
echo "Submitting incorrect quiz..."
RESULT=$(dfx canister call learning_engine submit_quiz '("1.0", vec { 0; 0 })')
echo "Result: $RESULT"

if [[ $RESULT == *"Incorrect answers"* ]]; then
    echo "PASS: Incorrect answers rejected."
else
    echo "FAIL: Incorrect answers not rejected properly."
    exit 1
fi

# Submit correct quiz
echo "Submitting correct quiz..."
RESULT=$(dfx canister call learning_engine submit_quiz '("1.0", vec { 1; 1 })')
echo "Result: $RESULT"

if [[ $RESULT == *"Ok"* ]]; then
    echo "PASS: Correct answers accepted."
else
    echo "FAIL: Correct answers rejected."
    exit 1
fi

# Verify completion status
echo "Verifying completion status..."
USER_ID=$(dfx identity get-principal)
RESULT=$(dfx canister call learning_engine is_quiz_completed "(principal \"$USER_ID\", \"1.0\")")
echo "Result: $RESULT"

if [[ $RESULT == *"true"* ]]; then
    echo "PASS: Quiz marked as completed."
else
    echo "FAIL: Quiz not marked as completed."
    exit 1
fi

echo "All tests passed!"
