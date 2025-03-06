use nlparsers::lrfg::{
    LRFGParser, Category, FConstraint, CNode, Rule,
    mapping::MappingRule, vocabulary::VocabularyItem
};

fn main() {
    println!("=== LEXICAL-REALIZATIONAL FUNCTIONAL GRAMMAR PARSER ===");
    
    // Create an LRFG parser with English grammar
    let mut parser = LRFGParser::new();
    parser.setup_english_grammar();
    
    // Enable debug mode
    parser.config.debug = true;
    parser.config.lfg_config.enforce_well_formedness = true;
    
    // Display some information about the grammar
    println!("\nGrammar information:");
    println!("Number of rules: {}", parser.lfg_parser.rules.len());
    println!("Lexicon entries: {}", parser.lfg_parser.lexicon.get_words().len());
    println!("Mapping rules: {}", parser.mapping.rules.len());
    println!("Vocabulary items: {}", parser.vocabulary.items.len());
    
    // Test sentences
    let sentences = [
        "the cat sleeps",
        "the cat sees the dog",
        "cat sleeps", // Testing NP -> N rule
        "the cats sleep", // Testing number agreement
        "the cat sleep", // Should fail due to number agreement violation
    ];
    
    println!("\nParsing sentences:");
    for sentence in &sentences {
        println!("\nSentence: {}", sentence);
        
        match parser.parse(sentence) {
            Some(parse_result) => {
                println!("✓ Valid parse found");
                
                // Print realized form
                if let Some(realized) = parse_result.realize() {
                    println!("Realized form: {}", realized);
                }
            },
            None => {
                println!("✗ No valid parse found");
            }
        }
    }
    
    // Demonstrate generation
    println!("\n=== GENERATION FROM F-STRUCTURE ===");
    
    // Parse a sentence to get its F-structure
    if let Some(parse_result) = parser.parse("the dog sees the cat") {
        if let Some(f_structure) = &parse_result.c_structure.f_structure {
            println!("\nOriginal F-structure:");
            println!("{}", f_structure);
            
            // Generate from this F-structure
            if let Some(generated) = parser.generate(f_structure) {
                println!("Generated: {}", generated);
            }
            
            // Modify the F-structure to change tense
            let mut modified_fs = f_structure.clone();
            modified_fs.set("TENSE", rust_grammar_formalisms::lfg::FValue::Atomic("past".to_string()));
            
            println!("\nModified F-structure (past tense):");
            println!("{}", modified_fs);
            
            // Generate from modified F-structure
            if let Some(generated) = parser.generate(&modified_fs) {
                println!("Generated: {}", generated);
            }
        }
    }
    
    // Demonstrate adding custom vocabulary items
    println!("\n=== ADDING CUSTOM VOCABULARY ITEMS ===");
    
    // Add a new vocabulary item for "house"
    let mut house_sg = VocabularyItem::new("house");
    house_sg.add_feature("pred", "house")
           .add_feature("num", "sg");
    parser.add_vocabulary_item(house_sg);
    
    let mut house_pl = VocabularyItem::new("houses");
    house_pl.add_feature("pred", "house")
           .add_feature("num", "pl");
    parser.add_vocabulary_item(house_pl);
    
    // Add lexical entry to the LFG parser
    let mut house_cat = Category::n();
    house_cat.add_equation("PRED", "'house'");
    parser.add_to_lexicon("house", house_cat);
    
    // Test with the new vocabulary
    let test_sentence = "the house";
    println!("\nTesting with new vocabulary: \"{}\"", test_sentence);
    
    if let Some(parse_result) = parser.parse(test_sentence) {
        println!("✓ Valid parse found");
        if let Some(realized) = parse_result.realize() {
            println!("Realized form: {}", realized);
        }
    } else {
        println!("✗ No valid parse found");
    }
    
    // Demonstrate adding a custom mapping rule
    println!("\n=== ADDING CUSTOM MAPPING RULES ===");
    
    // Add a mapping rule for definiteness
    let def_rule = MappingRule::with_value("SPEC TYPE", "def", "def", "yes");
    parser.add_mapping_rule(def_rule);
    
    // Add vocabulary item that uses definiteness
    let mut the_def = VocabularyItem::new("the");
    the_def.add_feature("cat", "Det")
          .add_feature("def", "yes")
          .with_priority(2); // Higher priority than existing "the" item
    parser.add_vocabulary_item(the_def);
    
    // Test with the new mapping rule
    println!("\nTesting with new mapping rule: \"the cat\"");
    
    if let Some(parse_result) = parser.parse("the cat") {
        println!("✓ Valid parse found");
        if let Some(realized) = parse_result.realize() {
            println!("Realized form: {}", realized);
        }
        
        // Show the R-structure with the new feature
        if let Some(r_structure) = &parse_result.r_structure {
            println!("R-structure with definiteness feature:");
            println!("{}", r_structure);
        }
    }
} 