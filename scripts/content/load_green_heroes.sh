#!/bin/bash

# ============================================================================
# Green Heroes - Content Loader
# Auto-generated from green-heroes-project.json
# ============================================================================

echo '📚 Loading Green Heroes content...'
echo '===================================='

dfx canister call learning_engine add_content_node '(record {
    id = "9887daaa-68b9-4406-a625-326851475989";
    parent_id = null;
    order = 0 : nat32;
    display_type = "BOOK";
    title = "Green Heroes";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "3334e941-aea7-4a43-ba02-36f0191776fa";
    parent_id = opt "9887daaa-68b9-4406-a625-326851475989";
    order = 0 : nat32;
    display_type = "CHAPTER";
    title = "Consumers, Be Aware and Beware!";
    description = opt "Some description about consumers to be aware of";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "14e22e9e-5a1c-41a9-9cd8-28c0baa660d6";
    parent_id = opt "9887daaa-68b9-4406-a625-326851475989";
    order = 1 : nat32;
    display_type = "SECTION";
    title = "Fast Fashion";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "ae1379c1-f003-4f6a-a414-d75504b0965e";
    parent_id = opt "9887daaa-68b9-4406-a625-326851475989";
    order = 2 : nat32;
    display_type = "CHAPTER";
    title = "Basics";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "9502bae4-956b-4add-aa79-c0016d84266f";
    parent_id = opt "9887daaa-68b9-4406-a625-326851475989";
    order = 3 : nat32;
    display_type = "CHAPTER";
    title = "Lifecycle Assessment (LCA)";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "a1332abd-6663-4816-82e2-37213230aa10";
    parent_id = opt "9887daaa-68b9-4406-a625-326851475989";
    order = 4 : nat32;
    display_type = "CHAPTER";
    title = "Misinformation Crisis";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "eb862071-527d-49bc-be45-bfe5afed212d";
    parent_id = opt "3334e941-aea7-4a43-ba02-36f0191776fa";
    order = 0 : nat32;
    display_type = "SECTION";
    title = "Unfortunate Consumers";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "69a81304-d8f5-4f26-89c9-fe22f72549d9";
    parent_id = opt "3334e941-aea7-4a43-ba02-36f0191776fa";
    order = 1 : nat32;
    display_type = "SECTION";
    title = "Microplastic vs. Natural Fiber Dust";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "d32a9e95-9830-4fe1-9f3a-2d63a6a677c4";
    parent_id = opt "3334e941-aea7-4a43-ba02-36f0191776fa";
    order = 2 : nat32;
    display_type = "SECTION";
    title = "Unfortunate Disposal";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "c4c889e9-771e-4fb2-9eb4-fe97a1eccc77";
    parent_id = opt "3334e941-aea7-4a43-ba02-36f0191776fa";
    order = 3 : nat32;
    display_type = "SECTION";
    title = "Unfortunate Recycling";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "3c41f520-0604-4c6a-8891-ee3474d346e4";
    parent_id = opt "14e22e9e-5a1c-41a9-9cd8-28c0baa660d6";
    order = 0 : nat32;
    display_type = "SUBSECTION";
    title = "2.1 Ambiguity in Definition";
    description = opt "Deconstruct the various definitions and concepts surrounding '\''fast fashion'\'' and examine how it drives consumer behavior and environmental impacts.";
    content = opt "Consumers are often misguided into accepting misinformation as scientifically valid truth about '\''fast fashion'\'' due to oversimplified definitions and perceptions.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "f561f8aa-9edc-4fd7-bb11-405c1e327035";
    parent_id = opt "14e22e9e-5a1c-41a9-9cd8-28c0baa660d6";
    order = 2 : nat32;
    display_type = "SUBSECTION";
    title = "2.2 Manufacturing Factors – Material, Quality and Cost";
    description = opt "Analyze the manufacturing aspects of fast fashion including materials, quality control, and cost-efficiency motivations that impact sustainability.";
    content = opt "Consumers perceptions of '\''quality'\'' in '\''fast fashion'\'' often conflict with scientific measures; synthetic fibers are generally stronger, implying contradictory perceptions.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "51a785b2-8705-4aab-b755-1e2646fde1da";
    parent_id = opt "14e22e9e-5a1c-41a9-9cd8-28c0baa660d6";
    order = 3 : nat32;
    display_type = "SUBSECTION";
    title = "2.3 Retail Price Factors";
    description = opt "Examine how retail pricing affects fast fashion trends and influences the perceived value versus environmental cost dilemma.";
    content = opt "Retail prices are shaped by operational structures and marketing, leading to a disconnect between actual material costs and price, influencing perception of quality in '\''fast fashion'\''.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "9d8f3748-a12b-43cb-bacd-67a8294e6c81";
    parent_id = opt "14e22e9e-5a1c-41a9-9cd8-28c0baa660d6";
    order = 4 : nat32;
    display_type = "SUBSECTION";
    title = "2.4 Misperceptions and Overconsumptions";
    description = opt "Assess consumer misunderstandings and the cultural trends that drive overconsumption in the fast fashion industry, together with environmental impacts.";
    content = opt "Labeling textiles as '\''fast fashion'\'' can lead to a perception of lower value and premature disposal, driving overconsumption, influenced by preconceived notions.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-5";
    parent_id = opt "14e22e9e-5a1c-41a9-9cd8-28c0baa660d6";
    order = 5 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Fast Fashion";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is one of the main issues with the term '\''fast fashion'\''?"; options = vec { "It is not a legally recognized term."; "Its definition is ambiguous and often leads to consumer misperception."; "It only applies to clothing, not accessories."; "It is a term invented by marketers." }; answer = 1 : nat8 }; record { question = "How do consumer perceptions of '\''quality'\'' in fast fashion often differ from scientific measures?"; options = vec { "Consumers often believe that natural fibers are always of higher quality, even though synthetic fibers can be stronger."; "Consumers believe that higher price always means higher quality."; "Consumers believe that items labeled '\''fast fashion'\'' are designed to be disposable."; "All of the above." }; answer = 3 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "f6c76051-ddb2-41c7-8488-7ca6146bf190";
    parent_id = opt "ae1379c1-f003-4f6a-a414-d75504b0965e";
    order = 0 : nat32;
    display_type = "SECTION";
    title = "Overview - Fibers";
    description = opt "An introductory exploration of different fiber types, their origins, applications, and role in the textile industry.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "b25c3a23-2c10-4e16-b342-3a3b5dd72713";
    parent_id = opt "ae1379c1-f003-4f6a-a414-d75504b0965e";
    order = 1 : nat32;
    display_type = "SECTION";
    title = "Natural Fibers";
    description = opt "Detailed exploration of natural fibers, including their properties, cultivation impacts, and applications in textiles.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "665ad822-1fea-4cb6-ba0e-138b9edbfd7f";
    parent_id = opt "ae1379c1-f003-4f6a-a414-d75504b0965e";
    order = 2 : nat32;
    display_type = "SECTION";
    title = "Synthetic fibers";
    description = opt "In-depth examination of synthetic fibers, including production methods, properties, and their significant role in the modern textile industry.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "602819c5-1c10-40fb-b2eb-c36c51c1979f";
    parent_id = opt "ae1379c1-f003-4f6a-a414-d75504b0965e";
    order = 3 : nat32;
    display_type = "SECTION";
    title = "Semi-Synthetic (also known as Semi-Natural) Fibers";
    description = opt "Explore the hybrid category of semi-synthetic fibers, illustrating how they bridge natural and synthetic materials.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "51d52a7d-6733-44fe-8d6c-bbe490f8276a";
    parent_id = opt "ae1379c1-f003-4f6a-a414-d75504b0965e";
    order = 4 : nat32;
    display_type = "SECTION";
    title = "Manufacturing";
    description = opt "Learn about the different manufacturing techniques used for fibers, their environmental implications, and advancements in textile sustainability.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "89e80112-58a3-4a5d-b867-2dc316d6cc4d";
    parent_id = opt "9502bae4-956b-4add-aa79-c0016d84266f";
    order = 0 : nat32;
    display_type = "SECTION";
    title = "LCA - Farming / Fiber Making";
    description = opt "Explore lifecycle assessment for natural and synthetic fiber manufacturing. Delve into initial production stages and their environmental effects.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "263e9803-2b53-40e2-9f6c-5de96beea581";
    parent_id = opt "9502bae4-956b-4add-aa79-c0016d84266f";
    order = 1 : nat32;
    display_type = "SECTION";
    title = "LCA – Manufacturing";
    description = opt "Investigate the ecological impacts of textile manufacturing processes and consider improvements for environmental sustainability.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "eda11401-d1c5-4908-b1ad-b9da02f7e5e8";
    parent_id = opt "9502bae4-956b-4add-aa79-c0016d84266f";
    order = 2 : nat32;
    display_type = "SECTION";
    title = "LCA - Consumption";
    description = opt "Analyze the environmental consequences related to textile consumption practices. Review informative data on consumer habits versus sustainability.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "74ad163e-1253-4d2c-9c7b-845ffbcbdd69";
    parent_id = opt "9502bae4-956b-4add-aa79-c0016d84266f";
    order = 3 : nat32;
    display_type = "SECTION";
    title = "LCA – Disposal";
    description = opt "Learn about the impact of textile disposal on the environment. Devise practices for minimizing waste and enhancing end-of-life sustainability.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "332b3c0f-fd58-4976-8109-1d2330a0599f";
    parent_id = opt "9502bae4-956b-4add-aa79-c0016d84266f";
    order = 4 : nat32;
    display_type = "SECTION";
    title = "LCA – Total Impacts";
    description = opt "Understand comprehensive perspectives on textile lifecycle impacts and discover cross-stage solutions to influence better practices.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-10";
    parent_id = opt "9502bae4-956b-4add-aa79-c0016d84266f";
    order = 5 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Lifecycle Assessment";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is the purpose of a Lifecycle Assessment (LCA)?"; options = vec { "To assess the financial cost of a product."; "To evaluate the environmental impacts of a product throughout its life cycle."; "To determine the market demand for a product."; "To certify a product as '\''eco-friendly'\''." }; answer = 1 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "d702bc77-4052-4f96-9568-dee686a74853";
    parent_id = opt "a1332abd-6663-4816-82e2-37213230aa10";
    order = 0 : nat32;
    display_type = "SECTION";
    title = "Greenwashing";
    description = opt "Identify, analyze, and overcome corporate greenwashing practices affecting public perception and sustainable consumer choices.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "43ec891c-3405-467f-b745-5f552731f59e";
    parent_id = opt "a1332abd-6663-4816-82e2-37213230aa10";
    order = 1 : nat32;
    display_type = "SECTION";
    title = "Self-Proclaimed Expertise";
    description = opt "Critically assess the rise of self-proclaimed experts in sustainability discussions and comprehend their influences on factual discourse.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "a387f17d-560d-47a3-8e8c-5ff3fb229f08";
    parent_id = opt "a1332abd-6663-4816-82e2-37213230aa10";
    order = 2 : nat32;
    display_type = "SECTION";
    title = "Organisations Lacking Expertise";
    description = opt "Understand the implications of organizations operating without necessary expertise, impacting sustainability initiatives and regulations.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "12ad502a-6b3c-41e1-8c83-c338f695d493";
    parent_id = opt "a1332abd-6663-4816-82e2-37213230aa10";
    order = 3 : nat32;
    display_type = "SECTION";
    title = "Loopholes in Certification Programs";
    description = opt "Explore how loopholes in certification affect consumer trust and sustainability in the global textile industry.";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-11";
    parent_id = opt "a1332abd-6663-4816-82e2-37213230aa10";
    order = 4 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Misinformation Crisis";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is '\''greenwashing'\''?"; options = vec { "A process for cleaning clothes with eco-friendly detergents."; "A marketing tactic to make a product appear more environmentally friendly than it actually is."; "A type of green dye used in the textile industry."; "A government initiative to promote sustainability." }; answer = 1 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "fcbb4f98-aa7b-413b-848a-a143ba3e970d";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 0 : nat32;
    display_type = "LEARNING POINT 1";
    title = "1.1 Material Choices and Care Methods";
    description = opt "Explore the misconceptions and environmental impacts regarding material choices between natural and synthetic fibers. Gain insight into how care methods affect textile longevity and sustainability.";
    content = opt "Origin Factor: This argument critically overlooks the significant energy consumption involved in farming natural fibers and, in some cases, the amount of the carbon footprint created in natural fibers can be multiple times higher than that of synthetic fibers.\nDecomposition Factor: The notion that natural fibers decompose quickly considers the breakdown of the natural elements like the cellulose of cotton and the protein of wool. However, it fails to account for the residual chemicals in textile waste.\nMicroplastic Factor: While synthetic fibers do release microplastics throughout their lifecycles, this argument critically overlooks the issues from natural fiber dust.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "7c80b704-376f-4ce4-b4a0-224be77f1832";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 1 : nat32;
    display_type = "LEARNING POINT";
    title = "1.1.1 Care Method Overview";
    description = opt "Understand the different care methods prescribed for textile goods, analyze the environmental impacts of these methods, and evaluate best practices for consumers.";
    content = opt "Material types can heavily influence the care frequencies during consumption period. For instance, cotton shirts and rags tend to develop odors more easily than their polyester counterparts, a phenomenon familiar to many. This observation aligns with the scientific understanding of the natural fibers and their chemical compositions, such as the presence of hydroxyl groups in the plant-based fibers (cotton, linen, etc.) and proteins in the animal-based fibers (wool, silk, etc.), which inherently offer high chemical reactivities with odor-causing microorganisms - A more detailed exploration on this topic is provided in Chapter II: Basics.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "ec46b838-02de-4465-b583-50f2152eb01c";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 2 : nat32;
    display_type = "SUBSECTION";
    title = "1.1.2 Natural Materials and Care Methods";
    description = opt "Detailed exploration of care methods specifically for natural materials like cotton, wool, and down, and their environmental implications.";
    content = opt "Numerous natural materials are used in textiles, primarily categorized as either animal- or plant-based. In this subsection, we will explore three of the most commonly used natural materials: Cotton, Wool and Down.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "d239616f-7650-4b94-8ce1-31deff9889cf";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 3 : nat32;
    display_type = "SUBSECTION";
    title = "1.1.3 Synthetic Materials and Care Methods";
    description = opt "Discover how care methods for synthetic materials differ from natural ones and how they influence environmental outcomes. Understand the strengths and limitations of synthetic materials in care.";
    content = opt "One significant advantage of most synthetic materials is that they typically do not require drycleaning. This distinction, compared to their natural counterparts, stems from a significantly lower chemical reactivity, which gives them a strong resistance to color fading. Furthermore, they offer much higher strength, enabling them to withstand physical forces encountered in home laundering with water.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-1";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 4 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Material Choices and Care Methods";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "According to the book, what is a common misconception about natural fibers?"; options = vec { "They are more expensive than synthetic fibers."; "They decompose quickly and without a trace."; "They are weaker than synthetic fibers."; "They require special washing techniques." }; answer = 1 : nat8 }; record { question = "Which of the following is NOT a factor to consider when evaluating the environmental impact of fibers?"; options = vec { "Origin Factor"; "Decomposition Factor"; "Microplastic Factor"; "Color Factor" }; answer = 3 : nat8 }; record { question = "Why do cotton shirts tend to develop odors more easily than polyester shirts?"; options = vec { "Because cotton is a natural fiber."; "Because of the presence of hydroxyl groups in cotton."; "Because polyester is a synthetic fiber."; "Because cotton is more absorbent." }; answer = 1 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "b63d7f19-9401-4eed-b4b0-759457e286b1";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 5 : nat32;
    display_type = "SUBSECTION";
    title = "1.2 Water Resistance & Breathable Technology";
    description = opt "Explanation and analysis of water resistance and breathability technologies. Learn the effectiveness and environmental impact of these technologies in real-world applications.";
    content = opt "\"Breathability\" refers to the fabric'\''s ability to allow moisture-laden air from the body to pass through (while providing a high level of water resistance in the context of this technology platform), providing dry comfort to wearers.The \"water resistant and breathable\" technology involves applying a polymeric membrane, typically made of the expanded-polytetrafluoroethylene (ePTFE), to the fabric'\''s backside (the side opposite to the weather-facing surface).";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "1e27278a-6bf2-42d5-b57f-923351309846";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 6 : nat32;
    display_type = "SUBSECTION";
    title = "1.3 Water-Repellency";
    description = opt "Explanation and analysis of water repellency technologies. Understand the chemical treatments used and their impact on the environment and fabric functionality.";
    content = opt "Water-repellency in textiles relates to the surface tension of fibers, fabric structures and their relationship with that of contacting water molecules. Per- and polyfluoroalkyl substances (PFAS) are applied to the weather-facing side of fabrics to create a high(er) degree of water repellency, preventing rain from wetting the surface.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-2";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 7 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Water Resistance and Repellency";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is the primary material used for the polymeric membrane in '\''water resistant and breathable'\'' technology?"; options = vec { "Polyvinyl Chloride (PVC)"; "Expanded-polytetrafluoroethylene (ePTFE)"; "Polyethylene terephthalate (PET)"; "Polystyrene (PS)" }; answer = 1 : nat8 }; record { question = "What substances are commonly applied to fabrics to achieve water-repellency?"; options = vec { "Silicones"; "Waxes"; "Per- and polyfluoroalkyl substances (PFAS)"; "Acrylics" }; answer = 2 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "6118ee45-7e7b-4f18-a535-3be0e638a28e";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 8 : nat32;
    display_type = "SUBSECTION";
    title = "1.4 Artificially Conceptual Value (ACV)";
    description = opt "Explore the concept of Artificially Conceptual Value (ACV), a phenomenon where technological features are perceived to be innovative yet lack proven scientific effectiveness. Understand the implications of ACVs on environmental sustainability and consumer perception, with examples from the textile industry.";
    content = opt "Artificially Conceptual Value (ACV) refers to a technical feature that lacks scientific evidence to support its effectiveness in real-life applications but is marketed in a way that convinces people it works. Companies engaged in promoting ACVs often introduce scientific explanations to justify their technologies, which may seem plausible to consumers without in-depth knowledge. This can lead to the adoption of technologies that do not contribute effectively to sustainability but are perceived to be valuable innovations. For example, heat reflective technology claims to reflect body heat to conserve warmth, but tests show minimal difference in thermal efficiency. Similarly, phase-change materials (PCMs) are marketed for their temperature-regulating properties, but the scale and activation conditions of PCMs often make them impractical for everyday textile applications. Furthermore, many performance claims associated with ACVs, such as health benefits from heat-generating technology, are unsupported when tested under realistic conditions. These ACVs not only mislead consumers but also pose environmental challenges, as unnecessary chemical treatments complicate recycling processes and contribute to environmental pollution.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 3 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "62905c65-59d5-4356-a6ea-04a66adc15aa";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 9 : nat32;
    display_type = "SUBSECTION";
    title = "1.4.1 Heat Reflective Technology";
    description = opt "Examine the science behind claimed heat reflective materials. Assess both the theoretical and real-life impact on consumer warmth and sustainability.";
    content = opt "A thin layer of a metallic silver pattern is applied to the inner lining fabric of winter jackets with the claim that it reflects the body heat back toward the wearer, conserving the heat and keeping him/her warmer than the person would be without this technology.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "97f0696a-4000-4c1e-b1d7-7d2d57784d75";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 10 : nat32;
    display_type = "SUBSECTION";
    title = "1.4.2 Phase-Change-Material (PCM) technology";
    description = opt "Explanation and analysis of Phase-Change-Material technology. Understand its intended thermal regulation purposes and the realistic outcome of textile applications.";
    content = opt "The PCM technology involves certain materials that change their phases from, for example, liquid to solid, and maintain a constant temperature during the phase change.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "83bfcd95-363d-476c-ac88-8ce8aa153d81";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 11 : nat32;
    display_type = "SUBSECTION";
    title = "1.4.3 Heat-Generating Technology";
    description = opt "Examine the claim of heat-generating textiles, focusing on the material scientific logic and environmental impact. Explore consumer bias toward perceived technology innovations.";
    content = opt "Extremely small particles of various materials including ceramic, clay, carbon nanotube, etc., are added to and evenly distributed in a polymer tank where the polymer is in a molten state.Fibers are then extruded with these particles embedded in them";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-3";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 12 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Artificially Conceptual Value";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is the definition of Artificially Conceptual Value (ACV)?"; options = vec { "A product that is genuinely innovative and scientifically proven."; "A marketing term for products that are environmentally friendly."; "A technical feature that is marketed as innovative but lacks scientific proof of effectiveness."; "A measure of a product'\''s market value." }; answer = 2 : nat8 }; record { question = "Which of these is an example of a technology that may have Artificially Conceptual Value?"; options = vec { "Heat reflective technology"; "Phase-Change-Material (PCM) technology"; "Heat-Generating Technology"; "All of the above" }; answer = 3 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "9553ed5a-d319-491a-85fe-89a52f8c2efd";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 13 : nat32;
    display_type = "SUBSECTION";
    title = "1.5 Other Important Textiles and Environmental Impacts";
    description = opt "Delve into complex textile issues affecting both consumer experience and environmental sustainability. Analyze examples that illustrate these impacts.";
    content = opt "In this subsection, I will explore a couple of textile technologies that can either greatly benefit or harm consumers while also having significant environmental impacts.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "ec373cc1-dddf-4dcd-88b0-c20997b39171";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 14 : nat32;
    display_type = "SUBSECTION";
    title = "1.5.1 Home Textile & Energy Savings";
    description = opt "Explore sustainable energy practices through home textiles, examining practices and innovations that contribute meaningfully to energy conservation.";
    content = opt "The thermally insulated curtains are a cost-effective and simple solution suitable for any homes or commercial buildings, regardless of the age of construction or window size and location. These curtains are made with specifically designed synthetic non-woven fabrics, which are engineered to provide optimal thermal efficiencies while using a minimal quantity of textile materials.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "2bc08326-903d-4d1d-888f-79ba74c23c5a";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 15 : nat32;
    display_type = "SUBSECTION";
    title = "1.5.2 Harmful Face Masks for Health and Environment";
    description = opt "Analyze the environmental and health misuse implications associated with face masks, weighing their necessity against sustainability techniques.";
    content = opt "Metal particles, such as copper and silver, cannot be permanently fixed to fibers and fabric structures...Placing such particles in the path of our breath is dangerous...Readers, please do not wear the face masks made with any particles regardless of its kind. Inhaling the particles can pose more harm than any potential benefit.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "e9d5f33f-0faf-4cb5-966e-2ebf059dbfd3";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 16 : nat32;
    display_type = "SECTION";
    title = "New Section";
    description = opt "";
    content = null;
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-4";
    parent_id = opt "eb862071-527d-49bc-be45-bfe5afed212d";
    order = 17 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Other Important Textiles and Environmental Impacts";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is the main benefit of thermally insulated curtains?"; options = vec { "They are decorative."; "They provide energy savings."; "They are made from natural materials."; "They are easy to clean." }; answer = 1 : nat8 }; record { question = "Why are face masks with metal particles considered harmful?"; options = vec { "They are not effective at filtering viruses."; "The metal particles can be inhaled."; "They are too expensive."; "They are bad for the environment." }; answer = 1 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "c8153ec8-01b3-4f60-a0ec-82b2000f5e58";
    parent_id = opt "69a81304-d8f5-4f26-89c9-fe22f72549d9";
    order = 0 : nat32;
    display_type = "SUBSECTION";
    title = "3.1 Microplastics and Synthetic Fibers";
    description = opt "Explore the ecological implications of synthetic fibers’ lifecycle, focusing on microplastic pollution and strategies for mitigation.";
    content = opt "Microplastics are generated from plastic sources, with synthetic fibers being a major source because they are weak during manufacturing.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "a1a90f3d-304e-4c0d-964d-bd54046a0f73";
    parent_id = opt "69a81304-d8f5-4f26-89c9-fe22f72549d9";
    order = 1 : nat32;
    display_type = "SUBSECTION";
    title = "3.2 Dust of Natural Fibers";
    description = opt "Evaluate natural fiber dust'\''s environmental effects and compare them with synthetic microplastics. Discover the common misconceptions and actual impact.";
    content = opt "Natural fibers, despite being '\''natural,'\'' also degrade and create dust particles that are harmful to both human and environmental health.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "5a4a6253-aaed-43b8-9480-496bca87a458";
    parent_id = opt "69a81304-d8f5-4f26-89c9-fe22f72549d9";
    order = 2 : nat32;
    display_type = "SUBSECTION";
    title = "3.2.1 Strength Factor and Dust Quantity";
    description = opt "Examine how the inherent strength of different fiber types affects dust production and impacts environmental health.";
    content = opt "Breaking energy analysis measures a fiber'\''s resistance to physical forces; polyester has higher values, implying cotton generates more dust.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-6";
    parent_id = opt "69a81304-d8f5-4f26-89c9-fe22f72549d9";
    order = 3 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Microplastic vs. Natural Fiber Dust";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is a significant source of microplastic pollution?"; options = vec { "Natural fibers"; "Synthetic fibers"; "Both natural and synthetic fibers"; "Neither" }; answer = 1 : nat8 }; record { question = "What does the '\''Strength Factor'\'' refer to in the context of fiber dust?"; options = vec { "The strength of the fabric."; "A measure of a fiber'\''s resistance to physical forces."; "The amount of dust a fiber produces."; "The colorfastness of a fiber." }; answer = 1 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "61a8737a-ab55-4782-8697-369bb0433a6d";
    parent_id = opt "d32a9e95-9830-4fe1-9f3a-2d63a6a677c4";
    order = 0 : nat32;
    display_type = "SUBSECTION";
    title = "4.1 Donation Myth";
    description = opt "Uncover the realities behind textile donation programs, understanding where donated clothing truly ends up and how it impacts the environment.";
    content = opt "Textile goods placed in recycling boxes or donated get shipped to underdeveloped countries for unethical practices, including burning or burying.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "b316a8f8-0a85-4cfe-925b-c65bfdf1c2c2";
    parent_id = opt "d32a9e95-9830-4fe1-9f3a-2d63a6a677c4";
    order = 1 : nat32;
    display_type = "SUBSECTION";
    title = "4.2 Decomposition Myth";
    description = opt "Analyze common misconceptions about textile decomposition timelines and processes in landfills.";
    content = opt "Natural fibers may degrade quicker, but chemicals persist, leading to other harmful effects from farming to natural existence, so it is crucial for us to emphasize chemical persistency.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "8fdc2500-36b6-43ce-bb6d-db0d86a0b3e6";
    parent_id = opt "d32a9e95-9830-4fe1-9f3a-2d63a6a677c4";
    order = 2 : nat32;
    display_type = "SUBSECTION";
    title = "4.2.1 Duration of Decomposition";
    description = opt "Explore the scientific and ecological aspects governing how long different materials take to decompose and the environmental impacts over time.";
    content = opt "Online sources state plastic can last 450 years in nature, though plastics do not undergo radioactive decay, so plastics are not chemically inert, and has the bacteria to degradate plastics.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "425912c8-5dc4-435b-a00a-4ff4cc83902d";
    parent_id = opt "d32a9e95-9830-4fe1-9f3a-2d63a6a677c4";
    order = 3 : nat32;
    display_type = "SUBSECTION";
    title = "4.3 Biodegradability";
    description = opt "Evaluate what biodegradability means in textiles, its merits, limits, and repercussions for sustainability strategies.";
    content = opt "Some textiles are labeled as '\''biodegradable'\'', but that term can be misleading, as they may just require specialized conditions. However, it is scientifically proven, after significant research, all plastics are biodegradable; however, not as effective as many natural materials such as wood.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-7";
    parent_id = opt "d32a9e95-9830-4fe1-9f3a-2d63a6a677c4";
    order = 4 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Unfortunate Disposal";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is the '\''Donation Myth'\''?"; options = vec { "That all donated clothes are sold in thrift stores."; "That donated clothes are always recycled."; "That donated clothes are often shipped to underdeveloped countries and disposed of unethically."; "That donating clothes is not helpful." }; answer = 2 : nat8 }; record { question = "What is the issue with the decomposition of natural fibers?"; options = vec { "They do not decompose."; "They decompose too quickly."; "The chemicals used in their production can persist in the environment."; "They release more microplastics than synthetic fibers." }; answer = 2 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "58283b70-2f38-4c2a-a0da-0660ff76bf0d";
    parent_id = opt "c4c889e9-771e-4fb2-9eb4-fe97a1eccc77";
    order = 0 : nat32;
    display_type = "SUBSECTION";
    title = "5. Unfortunate Recycling";
    description = opt "Explore the challenges and inefficiencies in the current textile recycling systems, and consider innovations that could make a meaningful impact.";
    content = opt "The global perspective is that purchasing recycled is good, but there are many different forms of measurements to ensure we can recycle without future implications.";
    paraphrase = null;
    media = null;
    quiz = null;
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 2 : nat64;
})'

dfx canister call learning_engine add_content_node '(record {
    id = "quiz-8";
    parent_id = opt "c4c889e9-771e-4fb2-9eb4-fe97a1eccc77";
    order = 1 : nat32;
    display_type = "QUIZ";
    title = "Quiz on Unfortunate Recycling";
    description = null;
    content = null;
    paraphrase = null;
    media = null;
    quiz = opt record { questions = vec { record { question = "What is a key challenge in textile recycling?"; options = vec { "A lack of consumer demand for recycled products."; "The difficulty in separating different fiber types."; "The high cost of recycling machinery."; "The lack of government support." }; answer = 1 : nat8 } } };
    created_at = 0 : nat64;
    updated_at = 0 : nat64;
    version = 1 : nat64;
})'


echo ''
echo '===================================='
echo '✅ Green Heroes content loaded successfully!'
echo ''
echo '📚 Summary:'
echo '   • Book: 1'
echo '   • Chapter: 4'
echo '   • Learning Point: 1'
echo '   • Learning Point 1: 1'
echo '   • Quiz: 10'
echo '   • Section: 20'
echo '   • Subsection: 23'
echo '   • Total nodes: 60'
echo '   • Quiz questions: 18'
echo ''
echo '📊 Verify with:'
echo '   dfx canister call learning_engine get_content_stats'
echo '   dfx canister call learning_engine get_root_nodes'
