#!/bin/bash

echo 'Loading sample learning units...'

dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "sample_1";
    unit_title = "Basic Colors";
    chapter_id = "sample_ch_1";
    chapter_title = "Sample Chapter 1: Basics";
    head_unit_id = "sample_head_1";
    head_unit_title = "Introduction to Colors";
    content = "There are three primary colors: Red, Blue, and Yellow. Mixing these colors creates secondary colors. For example, mixing Red and Blue makes Purple. Mixing Blue and Yellow makes Green. Mixing Red and Yellow makes Orange.";
    paraphrase = "# Colors\n\n- **Primary Colors**: Red, Blue, Yellow.\n- **Secondary Colors**:\n  - Red + Blue = Purple\n  - Blue + Yellow = Green\n  - Red + Yellow = Orange";
    quiz = vec { record { question = "What color do you get when you mix Red and Blue?"; options = vec { "Green"; "Purple"; "Orange"; "Black" }; answer = 1; }; record { question = "Which of these is a primary color?"; options = vec { "Green"; "Purple"; "Blue"; "Orange" }; answer = 2; }; };
})'

dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "sample_2";
    unit_title = "Simple Math";
    chapter_id = "sample_ch_1";
    chapter_title = "Sample Chapter 1: Basics";
    head_unit_id = "sample_head_2";
    head_unit_title = "Introduction to Math";
    content = "Addition is finding the total, or sum, by combining two or more numbers. Subtraction is taking one number away from another. Multiplication is repeated addition. Division is splitting into equal parts or groups.";
    paraphrase = "# Math Basics\n\n- **Addition**: Sum of numbers.\n- **Subtraction**: Taking away.\n- **Multiplication**: Repeated addition.\n- **Division**: Splitting into equal parts.";
    quiz = vec { record { question = "What is 2 + 2?"; options = vec { "3"; "4"; "5"; "6" }; answer = 1; }; record { question = "What is 5 - 3?"; options = vec { "1"; "2"; "3"; "4" }; answer = 1; }; };
})'

dfx canister call learning_engine add_learning_unit '(record {
    unit_id = "sample_3";
    unit_title = "Solar System";
    chapter_id = "sample_ch_2";
    chapter_title = "Sample Chapter 2: Science";
    head_unit_id = "sample_head_3";
    head_unit_title = "Space";
    content = "The Solar System consists of the Sun and the objects that orbit it. There are eight planets: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, and Neptune. The Sun is a star at the center of the Solar System.";
    paraphrase = "# The Solar System\n\n- **Center**: The Sun (a star).\n- **Planets**: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune.";
    quiz = vec { record { question = "Which planet is the third from the Sun?"; options = vec { "Mars"; "Venus"; "Earth"; "Jupiter" }; answer = 2; }; record { question = "What is at the center of the Solar System?"; options = vec { "Earth"; "The Moon"; "The Sun"; "Mars" }; answer = 2; }; };
})'

echo 'Done.'
