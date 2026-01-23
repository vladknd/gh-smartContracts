#!/usr/bin/env python3
"""
Generate a Big Book with Deep Hierarchical Structure
=====================================================
Creates a large book with multiple chapters, sections, subsections, 
and sub-subsections to stress-test the content governance system.

Structure:
- BOOK (root)
  └── PART (multiple)
      └── CHAPTER (multiple per part)
          └── SECTION (multiple per chapter)
              └── SUBSECTION (multiple per section)
                  └── UNIT (leaf nodes with content/quizzes)

Target: ~500+ content nodes to test:
- Staging capacity
- Proposal creation
- Chunked loading
- Quiz indexing

Usage:
    python3 generate_big_book.py [output_file.json]
"""

import json
import random
import hashlib
from datetime import datetime, timezone

# Configuration for book structure
CONFIG = {
    "num_parts": 5,           # Number of parts in the book
    "chapters_per_part": 4,   # Number of chapters per part  
    "sections_per_chapter": 3, # Number of sections per chapter
    "subsections_per_section": 3,  # Number of subsections
    "units_per_subsection": 2, # Leaf units with content
}

# Topic data for generating realistic content
TOPICS = {
    "parts": [
        ("Foundations of Environmental Science", "Understanding the basics of our planet"),
        ("Climate Systems and Change", "How Earth's climate works and changes"),
        ("Ecosystem Dynamics", "The complex world of living systems"),
        ("Human Impact and Sustainability", "Our effect on the environment"),
        ("Solutions and Future Outlook", "Addressing environmental challenges"),
    ],
    "chapter_themes": [
        "Introduction and Overview",
        "Key Concepts and Terminology", 
        "Historical Context",
        "Current State and Data",
        "Case Studies",
        "Advanced Topics",
        "Measurement and Analysis",
        "Regional Perspectives",
        "Policy and Governance",
        "Future Directions",
        "Interconnections",
        "Practical Applications",
        "Research Frontiers",
        "Global Perspectives",
        "Local Implications",
        "Emerging Trends",
        "Critical Analysis",
        "Synthesis and Integration",
        "Challenges and Opportunities",
        "Comparative Studies"
    ],
    "section_prefixes": [
        "Understanding", "Exploring", "Analyzing", "Examining", 
        "Investigating", "Studying", "Reviewing", "Assessing",
        "Evaluating", "Measuring", "Monitoring", "Comparing"
    ],
    "subsection_terms": [
        "Core Principles", "Fundamental Mechanisms", "Key Factors",
        "Primary Drivers", "Essential Components", "Critical Elements",
        "Main Processes", "Central Themes", "Important Aspects",
        "Significant Considerations", "Notable Features", "Major Influences"
    ]
}

# Quiz templates for generating varied questions
QUIZ_TEMPLATES = [
    {
        "template": "What is the primary characteristic of {topic}?",
        "options": ["Feature A", "Feature B", "Feature C", "Feature D"],
        "answer": 0
    },
    {
        "template": "Which factor most significantly affects {topic}?",
        "options": ["Factor X", "Factor Y", "Factor Z", "Factor W"],
        "answer": 1
    },
    {
        "template": "How does {topic} relate to environmental systems?",
        "options": ["Indirect relationship", "Direct relationship", "No relationship", "Complex relationship"],
        "answer": 3
    },
    {
        "template": "What is a key measurement for {topic}?",
        "options": ["Metric A", "Metric B", "Metric C", "Metric D"],
        "answer": 2
    }
]


def generate_id(prefix, *indices):
    """Generate a unique ID from prefix and indices."""
    parts = [prefix] + [str(i) for i in indices]
    return "_".join(parts)


def generate_content(topic, context, word_count=150):
    """Generate realistic-looking content text."""
    base_sentences = [
        f"{topic} is a critical aspect of {context}.",
        f"Understanding {topic} requires examining multiple interconnected factors.",
        f"Research has shown that {topic} plays a significant role in environmental systems.",
        f"The study of {topic} involves analyzing complex relationships between variables.",
        f"Environmental scientists have identified key patterns in {topic}.",
        f"Current data suggests that {topic} is influenced by both natural and human factors.",
        f"The importance of {topic} cannot be overstated in modern environmental science.",
        f"Practitioners in the field regularly monitor {topic} to track changes.",
        f"Historical records show that {topic} has evolved over time.",
        f"Models predicting {topic} continue to improve with better data.",
        f"The relationship between {topic} and climate is well documented.",
        f"Conservation efforts depend on accurate understanding of {topic}.",
        f"Policy decisions related to {topic} affect communities worldwide.",
        f"Technological advances have improved our ability to measure {topic}.",
        f"Education about {topic} is essential for environmental literacy.",
    ]
    
    # Shuffle and select sentences
    random.shuffle(base_sentences)
    content = " ".join(base_sentences[:8])
    return content


def generate_paraphrase(topic, context):
    """Generate a markdown-formatted paraphrase/summary."""
    return f"""# Summary: {topic}

**Key Points:**
- {topic} is essential for understanding {context}
- Multiple factors influence {topic} in complex ways
- Current research focuses on patterns and predictions
- Monitoring and measurement are critical for tracking changes

**Important Concepts:**
1. Fundamental principles of {topic}
2. Connections to broader environmental systems
3. Practical applications and implications
"""


def generate_quiz(topic, num_questions=2):
    """Generate quiz questions for a unit."""
    questions = []
    templates = random.sample(QUIZ_TEMPLATES, min(num_questions, len(QUIZ_TEMPLATES)))
    
    for template in templates:
        questions.append({
            "question": template["template"].format(topic=topic),
            "options": template["options"],
            "answer": template["answer"]
        })
    
    return {"questions": questions}


def generate_big_book():
    """Generate the complete hierarchical book structure."""
    nodes = []
    # Timestamp in nanoseconds (ICP uses nanoseconds since epoch)
    timestamp = int(datetime.now(timezone.utc).timestamp() * 1_000_000_000)
    run_id = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    # Root Book Node
    book_id = f"bigbook_{run_id}"
    nodes.append({
        "id": book_id,
        "parent_id": None,
        "order": 1,
        "display_type": "BOOK",
        "title": f"Comprehensive Environmental Science Encyclopedia ({run_id})",
        "description": "A large-scale test book with deep hierarchical structure for stress-testing content governance",
        "content": None,
        "paraphrase": None,
        "media": None,
        "quiz": None,
        "created_at": timestamp,
        "updated_at": timestamp,
        "version": 1
    })
    
    unit_counter = 0
    
    # Generate Parts
    for part_idx in range(CONFIG["num_parts"]):
        part_num = part_idx + 1
        part_id = generate_id(book_id, "part", part_num)
        part_title, part_desc = TOPICS["parts"][part_idx % len(TOPICS["parts"])]
        
        nodes.append({
            "id": part_id,
            "parent_id": book_id,
            "order": part_num,
            "display_type": "PART",
            "title": f"Part {part_num}: {part_title}",
            "description": part_desc,
            "content": None,
            "paraphrase": None,
            "media": None,
            "quiz": None,
            "created_at": timestamp,
            "updated_at": timestamp,
            "version": 1
        })
        
        # Generate Chapters within Part
        for ch_idx in range(CONFIG["chapters_per_part"]):
            ch_num = ch_idx + 1
            chapter_id = generate_id(part_id, "ch", ch_num)
            chapter_theme = TOPICS["chapter_themes"][(part_idx * CONFIG["chapters_per_part"] + ch_idx) % len(TOPICS["chapter_themes"])]
            
            nodes.append({
                "id": chapter_id,
                "parent_id": part_id,
                "order": ch_num,
                "display_type": "CHAPTER",
                "title": f"Chapter {part_num}.{ch_num}: {chapter_theme}",
                "description": f"Exploring {chapter_theme.lower()} in the context of {part_title.lower()}",
                "content": None,
                "paraphrase": None,
                "media": None,
                "quiz": None,
                "created_at": timestamp,
                "updated_at": timestamp,
                "version": 1
            })
            
            # Generate Sections within Chapter
            for sec_idx in range(CONFIG["sections_per_chapter"]):
                sec_num = sec_idx + 1
                section_id = generate_id(chapter_id, "sec", sec_num)
                section_prefix = TOPICS["section_prefixes"][(sec_idx) % len(TOPICS["section_prefixes"])]
                section_title = f"{section_prefix} {chapter_theme}"
                
                nodes.append({
                    "id": section_id,
                    "parent_id": chapter_id,
                    "order": sec_num,
                    "display_type": "SECTION",
                    "title": f"Section {part_num}.{ch_num}.{sec_num}: {section_title}",
                    "description": f"Detailed examination of {section_title.lower()}",
                    "content": None,
                    "paraphrase": None,
                    "media": None,
                    "quiz": None,
                    "created_at": timestamp,
                    "updated_at": timestamp,
                    "version": 1
                })
                
                # Generate Subsections within Section
                for subsec_idx in range(CONFIG["subsections_per_section"]):
                    subsec_num = subsec_idx + 1
                    subsection_id = generate_id(section_id, "subsec", subsec_num)
                    subsection_term = TOPICS["subsection_terms"][(subsec_idx) % len(TOPICS["subsection_terms"])]
                    
                    nodes.append({
                        "id": subsection_id,
                        "parent_id": section_id,
                        "order": subsec_num,
                        "display_type": "SUBSECTION",
                        "title": f"{subsection_term}: {section_prefix} Perspectives",
                        "description": f"Focused study on {subsection_term.lower()}",
                        "content": None,
                        "paraphrase": None,
                        "media": None,
                        "quiz": None,
                        "created_at": timestamp,
                        "updated_at": timestamp,
                        "version": 1
                    })
                    
                    # Generate Units (leaf nodes with actual content and quizzes)
                    for unit_idx in range(CONFIG["units_per_subsection"]):
                        unit_num = unit_idx + 1
                        unit_counter += 1
                        unit_id = generate_id(subsection_id, "unit", unit_num)
                        topic_name = f"{subsection_term} - Part {unit_num}"
                        context = f"{section_title} ({chapter_theme})"
                        
                        nodes.append({
                            "id": unit_id,
                            "parent_id": subsection_id,
                            "order": unit_num,
                            "display_type": "UNIT",
                            "title": f"Unit {unit_counter}: {topic_name}",
                            "description": f"Learning unit covering {topic_name.lower()}",
                            "content": generate_content(topic_name, context),
                            "paraphrase": generate_paraphrase(topic_name, context),
                            "media": None,
                            "quiz": generate_quiz(topic_name),
                            "created_at": timestamp,
                            "updated_at": timestamp,
                            "version": 1
                        })
    
    return {
        "metadata": {
            "name": f"Big Book Stress Test ({run_id})",
            "description": "Large hierarchical content structure for testing content governance proposal flow",
            "version": "1.0.0",
            "created_at": datetime.now(timezone.utc).isoformat(),
            "structure": CONFIG,
            "total_nodes": len(nodes),
            "hierarchy": ["BOOK", "PART", "CHAPTER", "SECTION", "SUBSECTION", "UNIT"],
            "total_units_with_quizzes": unit_counter
        },
        "content": nodes
    }


def calculate_stats(book_data):
    """Calculate statistics about the generated book."""
    nodes = book_data["content"]
    
    stats = {
        "total_nodes": len(nodes),
        "by_type": {},
        "with_content": 0,
        "with_quiz": 0,
        "total_quiz_questions": 0,
        "max_depth": 0,
        "avg_content_length": 0
    }
    
    content_lengths = []
    
    for node in nodes:
        display_type = node.get("display_type", "UNKNOWN")
        stats["by_type"][display_type] = stats["by_type"].get(display_type, 0) + 1
        
        if node.get("content"):
            stats["with_content"] += 1
            content_lengths.append(len(node["content"]))
        
        if node.get("quiz"):
            stats["with_quiz"] += 1
            stats["total_quiz_questions"] += len(node["quiz"].get("questions", []))
    
    if content_lengths:
        stats["avg_content_length"] = sum(content_lengths) / len(content_lengths)
    
    return stats


def main():
    import sys
    
    output_file = sys.argv[1] if len(sys.argv) > 1 else "big_book.json"
    
    print("=" * 70)
    print("  BIG BOOK GENERATOR - Content Governance Stress Test")
    print("=" * 70)
    print()
    
    print("Configuration:")
    for key, value in CONFIG.items():
        print(f"  {key}: {value}")
    print()
    
    print("Generating book structure...")
    book_data = generate_big_book()
    
    stats = calculate_stats(book_data)
    
    print("Statistics:")
    print(f"  Total Nodes: {stats['total_nodes']}")
    print(f"  Nodes by Type:")
    for display_type, count in sorted(stats["by_type"].items()):
        print(f"    - {display_type}: {count}")
    print(f"  Nodes with Content: {stats['with_content']}")
    print(f"  Nodes with Quizzes: {stats['with_quiz']}")
    print(f"  Total Quiz Questions: {stats['total_quiz_questions']}")
    print(f"  Average Content Length: {stats['avg_content_length']:.0f} characters")
    print()
    
    # Write to file
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(book_data, f, indent=2, ensure_ascii=False)
    
    file_size = len(json.dumps(book_data))
    print(f"Output written to: {output_file}")
    print(f"File size: {file_size / 1024:.1f} KB")
    print()
    print("=" * 70)
    print("  Book generation complete!")
    print("=" * 70)


if __name__ == "__main__":
    main()
