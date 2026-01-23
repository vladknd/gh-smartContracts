#!/bin/bash

# ============================================================================
# GHC Learning Platform - Test Curriculum Loader (Updated for ContentNode API)
# 5 Chapters, 3 Units Each = 15 Total Learning Units
# Easy questions with obvious answers for testing
# ============================================================================
# 
# IMPORTANT: This script now uses the new `add_content_node` API.
# The old `add_learning_unit` method has been deprecated.
#
# ContentNode Structure:
#   - id: unique identifier
#   - parent_id: parent node (for hierarchy), null for root
#   - order: display order
#   - display_type: "CHAPTER", "UNIT", "SECTION", etc.
#   - title: display title
#   - description: optional description
#   - content: main content text
#   - paraphrase: summary text
#   - media: optional media (null for text-only)
#   - quiz: optional quiz with questions
#   - created_at/updated_at: timestamps (0 for auto)
#   - version: content version
# ============================================================================

echo '🎓 Loading GHC Test Curriculum (ContentNode API)...'
echo '===================================='

# ============================================================================
# CHAPTER 1: World Geography
# ============================================================================
echo ''
echo '📍 Chapter 1: World Geography'

# Create Chapter Node
dfx canister call learning_engine add_content_node '(record {
    id = "chapter_1";
    parent_id = null;
    order = 1 : nat32;
    display_type = "CHAPTER";
    title = "World Geography";
    description = opt "Explore the world, its continents, oceans, and capitals.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 1.1
dfx canister call learning_engine add_content_node '(record {
    id = "1.1";
    parent_id = opt "chapter_1";
    order = 1 : nat32;
    display_type = "UNIT";
    title = "Major World Capitals";
    description = opt "Capitals of the World";
    content = opt "Every country has a capital city where the government is located. Paris is the capital of France, known for the Eiffel Tower. London is the capital of the United Kingdom, home to Big Ben. Tokyo is the capital of Japan, one of the largest cities in the world. Washington D.C. is the capital of the United States of America.";
    paraphrase = opt "World Capitals. France: Paris (Eiffel Tower). United Kingdom: London (Big Ben). Japan: Tokyo. USA: Washington D.C.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is the capital of France?"; options = vec { "London"; "Paris"; "Berlin"; "Madrid" }; answer = 1 : nat8 }; 
        record { question = "What is the capital of Japan?"; options = vec { "Beijing"; "Seoul"; "Tokyo"; "Bangkok" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 1.2
dfx canister call learning_engine add_content_node '(record {
    id = "1.2";
    parent_id = opt "chapter_1";
    order = 2 : nat32;
    display_type = "UNIT";
    title = "The Seven Continents";
    description = opt "Continents of Earth";
    content = opt "Earth has seven continents: Africa, Antarctica, Asia, Australia, Europe, North America, and South America. Africa is the second largest continent and home to the Sahara Desert. Asia is the largest continent, containing countries like China and India. Antarctica is the coldest continent, covered in ice.";
    paraphrase = opt "The Seven Continents. 1. Africa (Sahara Desert). 2. Antarctica (coldest). 3. Asia (largest). 4. Australia. 5. Europe. 6. North America. 7. South America.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "How many continents are on Earth?"; options = vec { "5"; "6"; "7"; "8" }; answer = 2 : nat8 }; 
        record { question = "Which is the largest continent?"; options = vec { "Africa"; "Asia"; "Europe"; "Australia" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 1.3
dfx canister call learning_engine add_content_node '(record {
    id = "1.3";
    parent_id = opt "chapter_1";
    order = 3 : nat32;
    display_type = "UNIT";
    title = "Oceans of the World";
    description = opt "Bodies of Water";
    content = opt "There are five oceans on Earth. The Pacific Ocean is the largest and deepest ocean. The Atlantic Ocean is the second largest. The Indian Ocean is the warmest. The Southern Ocean surrounds Antarctica. The Arctic Ocean is the smallest and coldest ocean, covered mostly by ice.";
    paraphrase = opt "The Five Oceans. 1. Pacific (largest, deepest). 2. Atlantic (second largest). 3. Indian (warmest). 4. Southern (around Antarctica). 5. Arctic (smallest, coldest).";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "Which is the largest ocean?"; options = vec { "Atlantic"; "Pacific"; "Indian"; "Arctic" }; answer = 1 : nat8 }; 
        record { question = "How many oceans are on Earth?"; options = vec { "3"; "4"; "5"; "6" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# ============================================================================
# CHAPTER 2: Animal Kingdom
# ============================================================================
echo '🦁 Chapter 2: Animal Kingdom'

# Create Chapter Node
dfx canister call learning_engine add_content_node '(record {
    id = "chapter_2";
    parent_id = null;
    order = 2 : nat32;
    display_type = "CHAPTER";
    title = "Animal Kingdom";
    description = opt "Learn about mammals, birds, and reptiles.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 2.1
dfx canister call learning_engine add_content_node '(record {
    id = "2.1";
    parent_id = opt "chapter_2";
    order = 1 : nat32;
    display_type = "UNIT";
    title = "Introduction to Mammals";
    description = opt "Types of Animals";
    content = opt "Mammals are warm-blooded animals that have hair or fur. Female mammals produce milk to feed their babies. Dogs, cats, elephants, whales, and humans are all mammals. The blue whale is the largest mammal on Earth. Bats are the only mammals that can truly fly.";
    paraphrase = opt "Mammals. Key Features: Warm-blooded, Have hair or fur, Females produce milk. Examples: Dogs, cats, elephants, whales, humans. Fun Facts: Blue whale is the largest mammal. Bats are the only flying mammals.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What do all mammals have?"; options = vec { "Hair or fur"; "Scales"; "Feathers"; "Shells" }; answer = 0 : nat8 }; 
        record { question = "What is the largest mammal on Earth?"; options = vec { "Elephant"; "Blue Whale"; "Giraffe"; "Polar Bear" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 2.2
dfx canister call learning_engine add_content_node '(record {
    id = "2.2";
    parent_id = opt "chapter_2";
    order = 2 : nat32;
    display_type = "UNIT";
    title = "Birds and Flight";
    description = opt "Feathered Friends";
    content = opt "Birds are animals with feathers and wings. Most birds can fly, but some like penguins and ostriches cannot. All birds lay eggs. The ostrich is the largest bird in the world. The hummingbird is one of the smallest birds and can fly backwards. Eagles are known for their excellent eyesight.";
    paraphrase = opt "Birds. Key Features: Have feathers and wings, Lay eggs, Most can fly. Records: Largest is Ostrich (cannot fly). Smallest is Hummingbird (can fly backwards). Best eyesight: Eagle.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is the largest bird in the world?"; options = vec { "Eagle"; "Penguin"; "Ostrich"; "Flamingo" }; answer = 2 : nat8 }; 
        record { question = "What do all birds lay?"; options = vec { "Seeds"; "Eggs"; "Milk"; "Honey" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 2.3
dfx canister call learning_engine add_content_node '(record {
    id = "2.3";
    parent_id = opt "chapter_2";
    order = 3 : nat32;
    display_type = "UNIT";
    title = "Reptiles and Scales";
    description = opt "Cold-Blooded Creatures";
    content = opt "Reptiles are cold-blooded animals covered in scales. They include snakes, lizards, turtles, and crocodiles. Reptiles lay eggs on land. They need the sun to warm their bodies because they cannot regulate their own body temperature. The Komodo dragon is the largest lizard in the world.";
    paraphrase = opt "Reptiles. Key Features: Cold-blooded, Covered in scales, Lay eggs on land. Examples: Snakes, lizards, turtles, crocodiles. Largest Lizard: Komodo Dragon.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What covers a reptile body?"; options = vec { "Feathers"; "Fur"; "Scales"; "Skin only" }; answer = 2 : nat8 }; 
        record { question = "Are reptiles warm-blooded or cold-blooded?"; options = vec { "Warm-blooded"; "Cold-blooded"; "Both"; "Neither" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# ============================================================================
# CHAPTER 3: Human Body
# ============================================================================
echo '🫀 Chapter 3: Human Body'

# Create Chapter Node
dfx canister call learning_engine add_content_node '(record {
    id = "chapter_3";
    parent_id = null;
    order = 3 : nat32;
    display_type = "CHAPTER";
    title = "Human Body";
    description = opt "Discover how your body works.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 3.1
dfx canister call learning_engine add_content_node '(record {
    id = "3.1";
    parent_id = opt "chapter_3";
    order = 1 : nat32;
    display_type = "UNIT";
    title = "The Human Heart";
    description = opt "Circulatory System";
    content = opt "The heart is a muscle that pumps blood throughout your body. It beats about 100,000 times per day. The heart has four chambers: two atria on top and two ventricles on the bottom. Blood carries oxygen and nutrients to all parts of the body. Red blood cells carry oxygen.";
    paraphrase = opt "The Heart. Function: Pumps blood through the body. Facts: Beats ~100,000 times per day. Has 4 chambers (2 atria + 2 ventricles). Blood: Carries oxygen and nutrients. Red blood cells transport oxygen.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "How many chambers does the heart have?"; options = vec { "2"; "3"; "4"; "5" }; answer = 2 : nat8 }; 
        record { question = "What does blood carry to the body?"; options = vec { "Oxygen"; "Sand"; "Air"; "Water only" }; answer = 0 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 3.2
dfx canister call learning_engine add_content_node '(record {
    id = "3.2";
    parent_id = opt "chapter_3";
    order = 2 : nat32;
    display_type = "UNIT";
    title = "Bones and Skeleton";
    description = opt "Skeletal System";
    content = opt "Adult humans have 206 bones in their body. Bones give our body structure and protect our organs. The skull protects the brain. The rib cage protects the heart and lungs. The femur (thigh bone) is the longest and strongest bone in the body. Calcium helps keep bones strong.";
    paraphrase = opt "The Skeleton. Adult Bones: 206 total. Functions: Gives body structure, Protects organs. Key Bones: Skull protects brain. Rib cage protects heart and lungs. Femur is longest and strongest bone. Tip: Calcium keeps bones strong!";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "How many bones does an adult human have?"; options = vec { "106"; "206"; "306"; "506" }; answer = 1 : nat8 }; 
        record { question = "What is the longest bone in the human body?"; options = vec { "Skull"; "Spine"; "Femur"; "Rib" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 3.3
dfx canister call learning_engine add_content_node '(record {
    id = "3.3";
    parent_id = opt "chapter_3";
    order = 3 : nat32;
    display_type = "UNIT";
    title = "The Amazing Brain";
    description = opt "Nervous System";
    content = opt "The brain is the control center of the body. It is protected by the skull. The brain controls everything you do: thinking, feeling, moving, and breathing. The brain is made of billions of nerve cells called neurons. The average adult brain weighs about 3 pounds. Sleep is essential for brain health.";
    paraphrase = opt "The Brain. Role: Control center of the body. Functions: Thinking and learning, Feeling emotions, Controlling movement, Breathing (automatic). Facts: Made of billions of neurons. Weighs ~3 pounds. Protected by the skull. Tip: Sleep is essential for brain health!";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What protects the brain?"; options = vec { "Rib cage"; "Spine"; "Skull"; "Skin" }; answer = 2 : nat8 }; 
        record { question = "What are brain cells called?"; options = vec { "Muscles"; "Neurons"; "Bones"; "Veins" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# ============================================================================
# CHAPTER 4: Space Exploration
# ============================================================================
echo '🚀 Chapter 4: Space Exploration'

# Create Chapter Node
dfx canister call learning_engine add_content_node '(record {
    id = "chapter_4";
    parent_id = null;
    order = 4 : nat32;
    display_type = "CHAPTER";
    title = "Space Exploration";
    description = opt "Journey through our solar system.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 4.1
dfx canister call learning_engine add_content_node '(record {
    id = "4.1";
    parent_id = opt "chapter_4";
    order = 1 : nat32;
    display_type = "UNIT";
    title = "Planets of Our Solar System";
    description = opt "Our Solar System";
    content = opt "Our solar system has 8 planets orbiting the Sun: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, and Neptune. Earth is the third planet from the Sun and the only known planet with life. Jupiter is the largest planet. Saturn is famous for its beautiful rings. Mars is called the Red Planet.";
    paraphrase = opt "The 8 Planets. Order from Sun: 1. Mercury (closest). 2. Venus. 3. Earth (has life!). 4. Mars (Red Planet). 5. Jupiter (largest). 6. Saturn (has rings!). 7. Uranus. 8. Neptune (farthest).";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "Which planet is known as the Red Planet?"; options = vec { "Jupiter"; "Venus"; "Mars"; "Saturn" }; answer = 2 : nat8 }; 
        record { question = "Which is the largest planet in our solar system?"; options = vec { "Earth"; "Saturn"; "Jupiter"; "Neptune" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 4.2
dfx canister call learning_engine add_content_node '(record {
    id = "4.2";
    parent_id = opt "chapter_4";
    order = 2 : nat32;
    display_type = "UNIT";
    title = "Earth Moon";
    description = opt "Our Natural Satellite";
    content = opt "The Moon is Earth natural satellite. It orbits around the Earth about once every 27 days. Neil Armstrong was the first person to walk on the Moon in 1969. The Moon has no atmosphere, so there is no wind or weather. We always see the same side of the Moon from Earth. The Moon causes ocean tides on Earth.";
    paraphrase = opt "The Moon. Facts: Earth natural satellite. Orbits Earth every ~27 days. No atmosphere (no wind/weather). Same side always faces Earth. Causes ocean tides. History: First human on Moon: Neil Armstrong (1969).";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "Who was the first person to walk on the Moon?"; options = vec { "Buzz Aldrin"; "Neil Armstrong"; "John Glenn"; "Yuri Gagarin" }; answer = 1 : nat8 }; 
        record { question = "What year did humans first land on the Moon?"; options = vec { "1959"; "1969"; "1979"; "1989" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 4.3
dfx canister call learning_engine add_content_node '(record {
    id = "4.3";
    parent_id = opt "chapter_4";
    order = 3 : nat32;
    display_type = "UNIT";
    title = "The Sun - Our Star";
    description = opt "Source of Light and Heat";
    content = opt "The Sun is a star at the center of our solar system. It is a giant ball of hot gas (mostly hydrogen and helium). The Sun provides light and heat that makes life on Earth possible. The Sun is about 93 million miles from Earth. Light from the Sun takes about 8 minutes to reach Earth. Never look directly at the Sun!";
    paraphrase = opt "The Sun. What is it? A star (giant ball of hot gas). Composition: Mostly hydrogen and helium. Distance: ~93 million miles from Earth. Light travel time: ~8 minutes to reach Earth. Importance: Provides light and heat for life. Never look directly at the Sun!";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is the Sun?"; options = vec { "A planet"; "A star"; "A moon"; "An asteroid" }; answer = 1 : nat8 }; 
        record { question = "What gas is the Sun mostly made of?"; options = vec { "Oxygen"; "Carbon"; "Hydrogen"; "Nitrogen" }; answer = 2 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# ============================================================================
# CHAPTER 5: Basic Mathematics
# ============================================================================
echo '🔢 Chapter 5: Basic Mathematics'

# Create Chapter Node
dfx canister call learning_engine add_content_node '(record {
    id = "chapter_5";
    parent_id = null;
    order = 5 : nat32;
    display_type = "CHAPTER";
    title = "Basic Mathematics";
    description = opt "Learn addition, subtraction, and multiplication.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 5.1
dfx canister call learning_engine add_content_node '(record {
    id = "5.1";
    parent_id = opt "chapter_5";
    order = 1 : nat32;
    display_type = "UNIT";
    title = "Addition Fundamentals";
    description = opt "Adding Numbers";
    content = opt "Addition is combining two or more numbers to find a total (sum). The symbol for addition is +. When you add 2 + 3, you get 5. When you add 10 + 10, you get 20. Addition can be done in any order (2 + 3 = 3 + 2 = 5). Zero added to any number gives the same number (5 + 0 = 5).";
    paraphrase = opt "Addition. Symbol: +. Examples: 2 + 3 = 5. 10 + 10 = 20. 5 + 0 = 5. Rules: Order does not matter (2+3 = 3+2). Adding zero keeps the number the same.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is 5 + 5?"; options = vec { "8"; "9"; "10"; "11" }; answer = 2 : nat8 }; 
        record { question = "What is 7 + 0?"; options = vec { "0"; "7"; "70"; "17" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 5.2
dfx canister call learning_engine add_content_node '(record {
    id = "5.2";
    parent_id = opt "chapter_5";
    order = 2 : nat32;
    display_type = "UNIT";
    title = "Subtraction Basics";
    description = opt "Taking Away Numbers";
    content = opt "Subtraction is taking one number away from another. The symbol for subtraction is -. When you subtract 5 - 2, you get 3. When you subtract 10 - 5, you get 5. Unlike addition, order matters in subtraction (5 - 2 is not the same as 2 - 5). Subtracting zero from any number gives the same number (8 - 0 = 8).";
    paraphrase = opt "Subtraction. Symbol: -. Examples: 5 - 2 = 3. 10 - 5 = 5. 8 - 0 = 8. Rules: Order DOES matter (5-2 is not 2-5). Subtracting zero keeps the number the same.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is 10 - 4?"; options = vec { "4"; "5"; "6"; "7" }; answer = 2 : nat8 }; 
        record { question = "What is 9 - 0?"; options = vec { "0"; "9"; "90"; "1" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

# Unit 5.3
dfx canister call learning_engine add_content_node '(record {
    id = "5.3";
    parent_id = opt "chapter_5";
    order = 3 : nat32;
    display_type = "UNIT";
    title = "Introduction to Multiplication";
    description = opt "Multiplying Numbers";
    content = opt "Multiplication is a faster way of adding the same number multiple times. The symbol for multiplication is x. For example, 3 x 4 means 3 + 3 + 3 + 3 = 12. Any number multiplied by 1 stays the same (5 x 1 = 5). Any number multiplied by 0 equals 0 (5 x 0 = 0). The 10 times table is easy: just add a zero (7 x 10 = 70).";
    paraphrase = opt "Multiplication. Symbol: x. Meaning: Repeated addition. 3 x 4 = 3 + 3 + 3 + 3 = 12. Rules: Any number x 1 = same number. Any number x 0 = 0. Any number x 10 = add a zero. Examples: 5 x 1 = 5. 5 x 0 = 0. 7 x 10 = 70.";
    media = null;
    quiz = opt record { questions = vec { 
        record { question = "What is 6 x 0?"; options = vec { "0"; "6"; "60"; "1" }; answer = 0 : nat8 }; 
        record { question = "What is 5 x 10?"; options = vec { "15"; "50"; "500"; "5" }; answer = 1 : nat8 }; 
    }};
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

echo ''
echo '===================================='
echo '✅ Test curriculum loaded successfully!'
echo ''
echo '📚 Summary:'
echo '   • Chapter 1: World Geography (3 units)'
echo '   • Chapter 2: Animal Kingdom (3 units)'
echo '   • Chapter 3: Human Body (3 units)'
echo '   • Chapter 4: Space Exploration (3 units)'
echo '   • Chapter 5: Basic Mathematics (3 units)'
echo ''
echo '   Total: 5 chapters + 15 learning units (20 content nodes)'
echo '         30 quiz questions (2 per unit)'
echo ''
echo '💡 Tip: All quiz answers are straightforward and based on the content!'
echo ''
echo '📊 Verify with:'
echo '   dfx canister call learning_engine get_content_stats'
echo '   dfx canister call learning_engine get_root_nodes'
