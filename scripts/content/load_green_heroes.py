#!/usr/bin/env python3
"""
Green Heroes Project Loader Generator

This script converts the green-heroes-project.json file (from AI Playground)
into a bash script that loads content nodes into the learning_engine canister.

Usage:
    python3 load_green_heroes.py
    
This will generate load_green_heroes.sh which can then be executed.
"""

import json
import os

# Path to the green-heroes-project.json file (in project root)
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
INPUT_FILE = os.path.join(PROJECT_ROOT, "green-heroes-project.json")
OUTPUT_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "load_green_heroes.sh")


def escape_candid_string(s):
    """Escape special characters for Candid/Bash strings."""
    if s is None:
        return None
    # Escape backslashes first
    s = s.replace('\\', '\\\\')
    # Escape double quotes for Candid string
    s = s.replace('"', '\\"')
    # Escape newlines
    s = s.replace('\n', '\\n')
    # Escape single quotes for Bash single-quoted string
    s = s.replace("'", "'\\''")
    return s


def format_optional(value, is_string=True):
    """Format a value as Candid optional."""
    if value is None:
        return "null"
    if is_string:
        escaped = escape_candid_string(value)
        return f'opt "{escaped}"'
    return f"opt {value}"


def format_quiz(quiz_data):
    """Format quiz data for Candid."""
    if quiz_data is None or 'questions' not in quiz_data or not quiz_data['questions']:
        return "null"
    
    questions = []
    for q in quiz_data['questions']:
        options_list = "; ".join([f'"{escape_candid_string(opt)}"' for opt in q.get('options', [])])
        options_vec = f"vec {{ {options_list} }}"
        answer = q.get('answer', 0)
        question_text = escape_candid_string(q.get('question', ''))
        questions.append(
            f'record {{ question = "{question_text}"; options = {options_vec}; answer = {answer} : nat8 }}'
        )
    
    questions_vec = "; ".join(questions)
    return f"opt record {{ questions = vec {{ {questions_vec} }} }}"


def convert_node(node):
    """Convert a node from AI Playground format to ContentNode format."""
    # Map the fields (AI Playground uses camelCase, canister uses snake_case)
    return {
        'id': node.get('id', ''),
        'parent_id': node.get('parentId'),
        'order': node.get('order', 0),
        'display_type': node.get('displayType', node.get('type', 'UNIT')).upper(),
        'title': node.get('title', ''),
        'description': node.get('description'),
        'content': node.get('content'),
        'paraphrase': None,  # AI Playground doesn't have this
        'media': None,  # TODO: handle media if needed
        'quiz': node.get('quiz'),
        'created_at': node.get('createdAt', 0),
        'updated_at': node.get('updatedAt', 0),
        'version': node.get('version', 1)
    }


def generate_dfx_call(node):
    """Generate a dfx canister call for a content node."""
    # Format all fields
    node_id = escape_candid_string(node['id'])
    parent_id = format_optional(node['parent_id'])
    order = node['order']
    display_type = escape_candid_string(node['display_type'])
    title = escape_candid_string(node['title'])
    description = format_optional(node['description'])
    content = format_optional(node['content'])
    paraphrase = format_optional(node['paraphrase'])
    quiz = format_quiz(node['quiz'])
    
    return f"""dfx canister call learning_engine add_content_node '(record {{
    id = "{node_id}";
    parent_id = {parent_id};
    order = {order} : nat32;
    display_type = "{display_type}";
    title = "{title}";
    description = {description};
    content = {content};
    paraphrase = {paraphrase};
    media = null;
    quiz = {quiz};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = {node['version']} : nat64;
}})'"""


def build_hierarchy(nodes_dict):
    """
    Build a proper loading order from the nodes dictionary.
    Returns nodes sorted so that parents are loaded before children.
    """
    nodes_list = list(nodes_dict.values())
    
    # Create a map for quick lookup
    id_to_node = {n['id']: n for n in nodes_list}
    
    # Find root nodes (no parent or parent not in the set)
    root_nodes = []
    child_nodes = []
    
    for node in nodes_list:
        parent_id = node.get('parentId')
        if parent_id is None or parent_id not in id_to_node:
            root_nodes.append(node)
        else:
            child_nodes.append(node)
    
    # Sort roots by order
    root_nodes.sort(key=lambda x: x.get('order', 0))
    
    # Build ordered list: BFS traversal to ensure parents loaded first
    ordered = []
    queue = root_nodes.copy()
    visited = set()
    
    while queue:
        current = queue.pop(0)
        node_id = current['id']
        
        if node_id in visited:
            continue
        visited.add(node_id)
        ordered.append(current)
        
        # Find children and add them
        children = [n for n in nodes_list if n.get('parentId') == node_id]
        children.sort(key=lambda x: x.get('order', 0))
        queue.extend(children)
    
    # Add any remaining nodes not reached (orphans)
    for node in nodes_list:
        if node['id'] not in visited:
            ordered.append(node)
    
    return ordered


def main():
    print(f"📚 Reading {INPUT_FILE}...")
    
    with open(INPUT_FILE, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    # Extract project info
    project_title = data.get('title', 'Unknown Project')
    nodes_dict = data.get('nodes', {})
    
    print(f"   Project: {project_title}")
    print(f"   Found {len(nodes_dict)} content nodes")
    
    # Build hierarchy and get ordered nodes
    ordered_nodes = build_hierarchy(nodes_dict)
    
    # Count by type
    type_counts = {}
    quiz_count = 0
    for node in ordered_nodes:
        display_type = node.get('displayType', node.get('type', 'unknown'))
        type_counts[display_type] = type_counts.get(display_type, 0) + 1
        if node.get('quiz'):
            quiz_count += len(node.get('quiz', {}).get('questions', []))
    
    print(f"\n📊 Content breakdown:")
    for t, count in sorted(type_counts.items()):
        print(f"   • {t}: {count}")
    print(f"   • Quiz questions: {quiz_count}")
    
    # Generate the loader script
    print(f"\n🔨 Generating {OUTPUT_FILE}...")
    
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        f.write("#!/bin/bash\n\n")
        f.write("# ============================================================================\n")
        f.write(f"# {project_title} - Content Loader\n")
        f.write("# Auto-generated from green-heroes-project.json\n")
        f.write("# ============================================================================\n\n")
        f.write(f"echo '📚 Loading {project_title} content...'\n")
        f.write("echo '===================================='\n\n")
        
        current_section = None
        for i, node in enumerate(ordered_nodes):
            converted = convert_node(node)
            display_type = converted['display_type']
            title = converted['title']
            
            # Add section headers for readability
            if display_type in ['SECTION', 'CHAPTER'] and node.get('parentId') is None:
                f.write(f"\n# ----------------------------------------------------------------------------\n")
                f.write(f"# {display_type}: {title}\n")
                f.write(f"# ----------------------------------------------------------------------------\n")
                f.write(f"echo '📖 Loading {display_type.lower()}: {title[:50]}...'\n\n")
            
            # Generate the dfx call
            call = generate_dfx_call(converted)
            f.write(call + "\n\n")
        
        f.write("\necho ''\n")
        f.write("echo '===================================='\n")
        f.write(f"echo '✅ {project_title} content loaded successfully!'\n")
        f.write("echo ''\n")
        f.write(f"echo '📚 Summary:'\n")
        for t, count in sorted(type_counts.items()):
            f.write(f"echo '   • {t}: {count}'\n")
        f.write(f"echo '   • Total nodes: {len(ordered_nodes)}'\n")
        f.write(f"echo '   • Quiz questions: {quiz_count}'\n")
        f.write("echo ''\n")
        f.write("echo '📊 Verify with:'\n")
        f.write("echo '   dfx canister call learning_engine get_content_stats'\n")
        f.write("echo '   dfx canister call learning_engine get_root_nodes'\n")
    
    # Make executable
    os.chmod(OUTPUT_FILE, 0o755)
    
    print(f"✅ Generated {OUTPUT_FILE}")
    print(f"\n🚀 To load the content, run:")
    print(f"   cd {os.path.dirname(OUTPUT_FILE)}")
    print(f"   ./load_green_heroes.sh")


if __name__ == "__main__":
    main()
