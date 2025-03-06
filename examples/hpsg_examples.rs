use std::rc::Rc;
use nlparsers::hpsg::{
    HPSGParser, Rule, RuleSchema, Category, Sign,
    HeadFeaturePrinciple, ValencePrinciple, FeatureStructure, TypedValue, FeatureType
};

/// Set up an English grammar for the HPSG parser
fn setup_english_grammar(parser: &mut HPSGParser) {
    // Set up basic grammar rules
    parser.setup_basic_grammar();
    
    // Add English-specific lexical entries
    let mut lexicon = &mut parser.lexicon;
    
    // Helper function to create a noun feature structure
    let create_noun_fs = |id: usize| {
        let mut fs = FeatureStructure::new("noun", id);
        
        fs.set("HEAD", TypedValue {
            type_name: "noun".to_string(),
            value: FeatureType::String("noun".to_string()),
            id: id + 1,
        });
        
        fs
    };
    
    // Helper function to create a verb feature structure
    let create_verb_fs = |id: usize| {
        let mut fs = FeatureStructure::new("verb", id);
        
        fs.set("HEAD", TypedValue {
            type_name: "verb".to_string(),
            value: FeatureType::String("verb".to_string()),
            id: id + 1,
        });
        
        fs
    };
    
    // Helper function to create a determiner feature structure
    let create_det_fs = |id: usize| {
        let mut fs = FeatureStructure::new("det", id);
        
        fs.set("HEAD", TypedValue {
            type_name: "det".to_string(),
            value: FeatureType::String("det".to_string()),
            id: id + 1,
        });
        
        fs
    };
    
    // Add some basic English words
    
    // Nouns
    lexicon.add_entry("cat", create_noun_fs(lexicon.next_id()));
    lexicon.add_entry("dog", create_noun_fs(lexicon.next_id()));
    lexicon.add_entry("book", create_noun_fs(lexicon.next_id()));
    lexicon.add_entry("man", create_noun_fs(lexicon.next_id()));
    lexicon.add_entry("woman", create_noun_fs(lexicon.next_id()));
    
    // Verbs
    lexicon.add_entry("sleeps", create_verb_fs(lexicon.next_id()));
    lexicon.add_entry("runs", create_verb_fs(lexicon.next_id()));
    lexicon.add_entry("sees", create_verb_fs(lexicon.next_id()));
    lexicon.add_entry("reads", create_verb_fs(lexicon.next_id()));
    
    // Determiners
    lexicon.add_entry("the", create_det_fs(lexicon.next_id()));
    lexicon.add_entry("a", create_det_fs(lexicon.next_id()));
    lexicon.add_entry("an", create_det_fs(lexicon.next_id()));
}

fn main() {
    println!("=== HEAD-DRIVEN PHRASE STRUCTURE GRAMMAR PARSER ===");
    
    // Create an HPSG parser
    let mut parser = HPSGParser::new();
    
    // Add principles
    let head_principle = Rc::new(HeadFeaturePrinciple::new());
    let valence_principle = Rc::new(ValencePrinciple::new());
    
    parser.add_principle(head_principle);
    parser.add_principle(valence_principle);
    
    // Set up the English grammar
    setup_english_grammar(&mut parser);
    
    // Print information about the grammar
    println!("\nGrammar Information:");
    println!("Number of rules: {}", parser.rules.len());
    println!("Lexicon size: {} words", parser.lexicon.len());
    
    println!("\nRules:");
    for (i, rule) in parser.rules.iter().enumerate() {
        println!("  {}: {}", i + 1, rule.name);
    }
    
    // Test sentences
    let sentences = [
        "the cat sleeps",
        "the dog sees the cat",
        "the man reads the book",
        "cat sleeps", // Testing just N V
    ];
    
    println!("\nParsing sentences:");
    for sentence in &sentences {
        println!("\nSentence: {}", sentence);
        
        // Parse the sentence and get analyses
        let analyses = parser.parse_internal(sentence);
        
        if analyses.is_empty() {
            println!("✗ No valid parse found");
        } else {
            println!("✓ Found {} analyses", analyses.len());
            
            // Print the first analysis
            let analysis = &analyses[0];
            println!("\nFirst analysis:");
            println!("Type: {}", analysis.sign_type);
            println!("Phonetic form: {}", analysis.phonetic_form());
            
            // Print the head
            if let Some(head) = analysis.head_daughter() {
                println!("Head daughter: {}", head.phonetic_form());
            }
            
            // Print the feature structure (simplified)
            println!("\nFeature structure (simplified):");
            print_features(&analysis.feature_structure);
            
            // Print the tree structure
            println!("\nTree structure:");
            print_tree(analysis, 0);
        }
    }
    
    // Demonstrate adding a custom lexical entry
    println!("\n=== Adding a Custom Lexical Entry ===");
    
    let mut verb_fs = FeatureStructure::new("verb", parser.next_sign_index());
    verb_fs.set("HEAD", TypedValue {
        type_name: "verb".to_string(),
        value: FeatureType::String("verb".to_string()),
        id: parser.next_sign_index(),
    });
    
    parser.lexicon.add_entry("jumps", verb_fs);
    
    println!("Added 'jumps' to the lexicon as a verb");
    
    // Test the new entry
    let analyses = parser.parse_internal("the cat jumps");
    
    if analyses.is_empty() {
        println!("✗ No valid parse found for 'the cat jumps'");
    } else {
        println!("✓ Valid parse found for 'the cat jumps'");
        println!("Phonetic form: {}", analyses[0].phonetic_form());
    }
}

/// Helper function to print features in a readable format
fn print_features(fs: &FeatureStructure) {
    for (key, value) in &fs.features {
        println!("  {}: {}", key, value);
    }
}

/// Helper function to print the tree structure
fn print_tree(sign: &Sign, indent: usize) {
    let indent_str = "  ".repeat(indent);
    
    println!("{}{} [{}]", indent_str, sign.phonetic_form(), sign.sign_type);
    
    for daughter in &sign.daughters {
        print_tree(daughter, indent + 1);
    }
}