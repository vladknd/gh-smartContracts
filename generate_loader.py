import json

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

def generate_loader_script(json_file, output_file):
    with open(json_file, 'r') as f:
        units = json.load(f)

    with open(output_file, 'w') as f:
        f.write("#!/bin/bash\n\n")
        f.write("echo 'Loading learning units...'\n\n")
        
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
    generate_loader_script('learning_materials.json', 'load_data.sh')
