#!/bin/bash

# Setup
set -e
echo "Starting limits test..."

# 1. Add Content
# We add a dummy unit with a quiz to the learning_engine.
# Note: This relies on add_content_node being accessible (admin or open in dev).
echo "Adding test unit 'test_unit_limit'..."
dfx canister call learning_engine add_content_node '(record {
  id = "test_unit_limit";
  parent_id = null;
  order = 1000;
  display_type = "UNIT";
  title = "Limit Test Unit";
  description = null;
  content = null;
  paraphrase = null;
  media = null;
  quiz = opt record {
    questions = vec {
      record {
        question = "Q1?";
        options = vec { "Right"; "Wrong" };
        answer = 0;
      }
    }
  };
  created_at = 0;
  updated_at = 0;
  version = 0;
})'

# 2. Register User
# We attempt to register. If already registered, we ignore the specific error but check output if needed.
echo "Registering user..."
# We use || true because if user is already registered it might trap or return Err, 
# but we want to proceed.
dfx canister call user_profile register_user '(record {
  email = "test@example.com";
  name = "Test User";
  education = "None";
  gender = "None";
})' || echo "User likely already registered."

# 3. Submit 5 wrong answers (Attempts)
# Global limit is 5 attempts per day.
# We submit wrong answer (index 1) to ensure we don't 'complete' the quiz 
# (though limits apply to attempts anyway, failing keeps it simpler).
echo "Submitting 5 attempts..."
for i in {1..5}; do
  echo "Attempt $i..."
  OUTPUT=$(dfx canister call user_profile submit_quiz '("test_unit_limit", vec {1})')
  echo "Result: $OUTPUT"
  
  # Optional: Check if it counted as an attempt (it should, returning low score error)
  if [[ "$OUTPUT" != *"Quiz failed"* ]] && [[ "$OUTPUT" != *"Ok"* ]]; then
     echo "Unexpected output on attempt $i"
  fi
done

# 4. Attempt 6 (Should be blocked)
echo "Attempt 6 (Expect: Daily quiz limit reached)..."
OUTPUT=$(dfx canister call user_profile submit_quiz '("test_unit_limit", vec {1})')
echo "Result: $OUTPUT"

if [[ "$OUTPUT" == *"Daily quiz limit reached"* ]]; then
  echo "✅ SUCCESS: Daily limit reached as expected."
else
  echo "❌ FAILURE: Expected 'Daily quiz limit reached', got '$OUTPUT'"
  exit 1
fi
