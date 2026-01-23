#!/bin/bash

# ============================================================================
# GHC Learning Platform - Sample Data Loader (Updated for ContentNode API)
# Quick 3-unit sample for testing
# ============================================================================

echo 'Loading sample learning units (ContentNode API)...'

# Sample Chapter
dfx canister call learning_engine add_content_node '(record {
    id = "sample_chapter";
    parent_id = null;
    order = 100 : nat32;
    display_type = "CHAPTER";
    title = "Sample Chapter: Quick Tests";
    description = opt "Sample content for quick testing.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Sample Unit 1: Colors
dfx canister call learning_engine add_content_node '(record {
    id = "sample_1";
    parent_id = opt "sample_chapter";
    order = 1 : nat32;
    display_type = "UNIT";
    title = "Basic Colors";
    description = opt "Introduction to Colors";
    content = opt "There are three primary colors: Red, Blue, and Yellow. Mixing these colors creates secondary colors. For example, mixing Red and Blue makes Purple. Mixing Blue and Yellow makes Green. Mixing Red and Yellow makes Orange.";
    paraphrase = opt "Primary Colors: Red, Blue, Yellow. Secondary Colors: Red + Blue = Purple, Blue + Yellow = Green, Red + Yellow = Orange.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What color do you get when you mix Red and Blue?"; options = vec { "Green"; "Purple"; "Orange"; "Black" }; answer = 1 : nat8 }; 
        record { question = "Which of these is a primary color?"; options = vec { "Green"; "Purple"; "Blue"; "Orange" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Sample Unit 2: Math
dfx canister call learning_engine add_content_node '(record {
    id = "sample_2";
    parent_id = opt "sample_chapter";
    order = 2 : nat32;
    display_type = "UNIT";
    title = "Simple Math";
    description = opt "Introduction to Math";
    content = opt "Addition is finding the total, or sum, by combining two or more numbers. Subtraction is taking one number away from another. Multiplication is repeated addition. Division is splitting into equal parts or groups.";
    paraphrase = opt "Addition: Sum of numbers. Subtraction: Taking away. Multiplication: Repeated addition. Division: Splitting into equal parts.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is 2 + 2?"; options = vec { "3"; "4"; "5"; "6" }; answer = 1 : nat8 }; 
        record { question = "What is 5 - 3?"; options = vec { "1"; "2"; "3"; "4" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Sample Unit 3: Space
dfx canister call learning_engine add_content_node '(record {
    id = "sample_3";
    parent_id = opt "sample_chapter";
    order = 3 : nat32;
    display_type = "UNIT";
    title = "Solar System";
    description = opt "Space";
    content = opt "The Solar System consists of the Sun and the objects that orbit it. There are eight planets: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, and Neptune. The Sun is a star at the center of the Solar System.";
    paraphrase = opt "Center: The Sun (a star). Planets: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "Which planet is the third from the Sun?"; options = vec { "Mars"; "Venus"; "Earth"; "Jupiter" }; answer = 2 : nat8 }; 
        record { question = "What is at the center of the Solar System?"; options = vec { "Earth"; "The Moon"; "The Sun"; "Mars" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

echo ''
echo 'Done! Loaded 1 chapter + 3 units with 6 quiz questions.'
echo ''
echo 'Verify with:'
echo '  dfx canister call learning_engine get_content_stats'
echo '  dfx canister call learning_engine get_children '\''("sample_chapter")'\'''
