import json
import os

def escape_candid_string(s):
    # Escape backslashes first to avoid escaping the escapes
    s = s.replace('\\', '\\\\')
    # Escape double quotes for Candid string
    s = s.replace('"', '\\"')
    # Escape newlines
    s = s.replace('\n', '\\n')
    # Escape single quotes for Bash single-quoted string
    s = s.replace("'", "'\\''")
    return s

def generate_sample_loader_script(output_file):
    # Sample Data
    units = [
        {
            "unit_id": "sample_1",
            "unit_title": "Basic Colors",
            "chapter_id": "sample_ch_1",
            "chapter_title": "Sample Chapter 1: Basics",
            "head_unit_id": "sample_head_1",
            "head_unit_title": "Introduction to Colors",
            "content": "There are three primary colors: Red, Blue, and Yellow. Mixing these colors creates secondary colors. For example, mixing Red and Blue makes Purple. Mixing Blue and Yellow makes Green. Mixing Red and Yellow makes Orange.",
            "paraphrase": "# Colors\n\n- **Primary Colors**: Red, Blue, Yellow.\n- **Secondary Colors**:\n  - Red + Blue = Purple\n  - Blue + Yellow = Green\n  - Red + Yellow = Orange",
            "quiz": [
                {
                    "question": "What color do you get when you mix Red and Blue?",
                    "options": ["Green", "Purple", "Orange", "Black"],
                    "answer": 1
                },
                {
                    "question": "Which of these is a primary color?",
                    "options": ["Green", "Purple", "Blue", "Orange"],
                    "answer": 2
                }
            ]
        },
        {
            "unit_id": "sample_2",
            "unit_title": "Simple Math",
            "chapter_id": "sample_ch_1",
            "chapter_title": "Sample Chapter 1: Basics",
            "head_unit_id": "sample_head_2",
            "head_unit_title": "Introduction to Math",
            "content": "Addition is finding the total, or sum, by combining two or more numbers. Subtraction is taking one number away from another. Multiplication is repeated addition. Division is splitting into equal parts or groups.",
            "paraphrase": "# Math Basics\n\n- **Addition**: Sum of numbers.\n- **Subtraction**: Taking away.\n- **Multiplication**: Repeated addition.\n- **Division**: Splitting into equal parts.",
            "quiz": [
                {
                    "question": "What is 2 + 2?",
                    "options": ["3", "4", "5", "6"],
                    "answer": 1
                },
                {
                    "question": "What is 5 - 3?",
                    "options": ["1", "2", "3", "4"],
                    "answer": 1
                }
            ]
        },
        {
            "unit_id": "sample_3",
            "unit_title": "Solar System",
            "chapter_id": "sample_ch_2",
            "chapter_title": "Sample Chapter 2: Science",
            "head_unit_id": "sample_head_3",
            "head_unit_title": "Space",
            "content": "The Solar System consists of the Sun and the objects that orbit it. There are eight planets: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, and Neptune. The Sun is a star at the center of the Solar System.",
            "paraphrase": "# The Solar System\n\n- **Center**: The Sun (a star).\n- **Planets**: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune.",
            "quiz": [
                {
                    "question": "Which planet is the third from the Sun?",
                    "options": ["Mars", "Venus", "Earth", "Jupiter"],
                    "answer": 2
                },
                {
                    "question": "What is at the center of the Solar System?",
                    "options": ["Earth", "The Moon", "The Sun", "Mars"],
                    "answer": 2
                }
            ]
        }
    ]

    with open(output_file, 'w') as f:
        f.write("#!/bin/bash\n\n")
        f.write("echo 'Loading sample learning units...'\n\n")
        
        for unit in units:
            # Construct quiz vector
            quiz_vec = "vec {"
            for q in unit['quiz']:
                options_vec = "vec { " + "; ".join([f'"{escape_candid_string(opt)}"' for opt in q['options']]) + " }"
                quiz_vec += f' record {{ question = "{escape_candid_string(q["question"])}"; options = {options_vec}; answer = {q["answer"]}; }};'
            quiz_vec += " }"

            # Construct the call
            cmd = f"""dfx canister call learning_engine add_learning_unit '(record {{
    unit_id = "{unit['unit_id']}";
    unit_title = "{escape_candid_string(unit['unit_title'])}";
    chapter_id = "{unit['chapter_id']}";
    chapter_title = "{escape_candid_string(unit['chapter_title'])}";
    head_unit_id = "{unit['head_unit_id']}";
    head_unit_title = "{escape_candid_string(unit['head_unit_title'])}";
    content = "{escape_candid_string(unit['content'])}";
    paraphrase = "{escape_candid_string(unit['paraphrase'])}";
    quiz = {quiz_vec};
}})'"""
            f.write(cmd + "\n\n")
            
        f.write("echo 'Done.'\n")

if __name__ == "__main__":
    generate_sample_loader_script('load_sample_data.sh')
