use nlparsers::mg::{
    Feature, LexicalItem, MinimalistParser, ParserConfig
};
use nlparsers::common::FeatureStructure;
use nlparsers::common::Parser;

fn main() {
    println!("=== MINIMALIST GRAMMAR PARSER ===");
    
    // English examples
    println!("\n--- ENGLISH EXAMPLES ---");
    let english_parser = setup_english_parser();
    
    let english_sentences = [
        "the cat sleeps",
        "the dog chases the cat",
        "John will see Mary",
        "the man who loves Mary sleeps",
    ];
    
    for sentence in &english_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = english_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Wh-movement examples
    println!("\n--- WH-MOVEMENT EXAMPLES ---");
    let wh_parser = setup_wh_parser();
    
    let wh_sentences = [
        "what John saw",
        "who saw Mary",
        "which book John read",
    ];
    
    for sentence in &wh_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = wh_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Head-final language examples (Japanese-like)
    println!("\n--- HEAD-FINAL LANGUAGE EXAMPLES ---");
    let head_final_parser = setup_head_final_parser();
    
    let head_final_sentences = [
        "John Mary saw",
        "cat slept",
        "John Mary book gave",
    ];
    
    for sentence in &head_final_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = head_final_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Agreement examples
    println!("\n--- AGREEMENT EXAMPLES ---");
    let agreement_parser = setup_agreement_parser();
    
    let agreement_sentences = [
        "the cat sleeps",
        "the cats sleep",
        "a cat sleeps",
        "some cats sleep",
        "the cat sleep",  // Ungrammatical: agreement violation
        "some cat sleeps" // Ungrammatical: agreement violation
    ];
    
    for sentence in &agreement_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = agreement_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Phase-based examples
    println!("\n--- PHASE-BASED EXAMPLES ---");
    let phase_parser = setup_phase_parser();
    
    let phase_sentences = [
        "John thinks that Mary saw Bill",
        "what does John think that Mary saw",
    ];
    
    for sentence in &phase_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = phase_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
}


// Setup a basic English Minimalist Grammar parser
fn setup_english_parser() -> MinimalistParser {
    let mut parser = MinimalistParser::new();
    
    // Determiners
    parser.add_to_lexicon("the", LexicalItem::new("the", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
    ]));
    
    parser.add_to_lexicon("a", LexicalItem::new("a", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
    ]));
    
    // Nouns
    parser.add_to_lexicon("cat", LexicalItem::new("cat", vec![
        Feature::categorial("N"),
    ]));
    
    parser.add_to_lexicon("dog", LexicalItem::new("dog", vec![
        Feature::categorial("N"),
    ]));
    
    parser.add_to_lexicon("man", LexicalItem::new("man", vec![
        Feature::categorial("N"),
    ]));
    
    // Proper names
    parser.add_to_lexicon("John", LexicalItem::new("John", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("Mary", LexicalItem::new("Mary", vec![
        Feature::categorial("D"),
    ]));
    
    // Intransitive verbs
    parser.add_to_lexicon("sleeps", LexicalItem::new("sleeps", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject
    ]));
    
    parser.add_to_lexicon("runs", LexicalItem::new("runs", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject
    ]));
    
    // Transitive verbs
    parser.add_to_lexicon("chases", LexicalItem::new("chases", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    parser.add_to_lexicon("sees", LexicalItem::new("sees", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    parser.add_to_lexicon("loves", LexicalItem::new("loves", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    // Auxiliaries
    parser.add_to_lexicon("will", LexicalItem::new("will", vec![
        Feature::categorial("T"),
        Feature::selector("V"),
        Feature::selector("D"),  // Subject
    ]));
    
    // Relative pronouns
    parser.add_to_lexicon("who", LexicalItem::new("who", vec![
        Feature::categorial("C"),
        Feature::selector("T"),
        Feature::licensee("wh"),  // For movement
    ]));
    
    // Complementizer
    parser.add_to_lexicon("that", LexicalItem::new("that", vec![
        Feature::categorial("C"),
        Feature::selector("T"),
    ]));
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.max_derivation_depth = 20;
    config.allow_remnant_movement = true;
    parser.set_config(config);
    
    parser
}

// Setup a parser for wh-movement
fn setup_wh_parser() -> MinimalistParser {
    let mut parser = MinimalistParser::new();
    
    // Wh-words
    parser.add_to_lexicon("what", LexicalItem::new("what", vec![
        Feature::categorial("D"),
        Feature::licensee("wh"),  // For wh-movement
    ]));
    
    parser.add_to_lexicon("who", LexicalItem::new("who", vec![
        Feature::categorial("D"),
        Feature::licensee("wh"),  // For wh-movement
    ]));
    
    parser.add_to_lexicon("which", LexicalItem::new("which", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
        Feature::licensee("wh"),  // For wh-movement
    ]));
    
    // Nouns
    parser.add_to_lexicon("book", LexicalItem::new("book", vec![
        Feature::categorial("N"),
    ]));
    
    // Proper names
    parser.add_to_lexicon("John", LexicalItem::new("John", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("Mary", LexicalItem::new("Mary", vec![
        Feature::categorial("D"),
    ]));
    
    // Verbs
    parser.add_to_lexicon("saw", LexicalItem::new("saw", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    parser.add_to_lexicon("read", LexicalItem::new("read", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    // C with wh-feature to attract wh-phrases
    parser.add_to_lexicon("C", LexicalItem::new("", vec![
        Feature::categorial("C"),
        Feature::licensor("wh"),  // Attracts wh-phrases
        Feature::selector("T"),
    ]));
    
    // Silent T
    parser.add_to_lexicon("T", LexicalItem::new("", vec![
        Feature::categorial("T"),
        Feature::selector("V"),
    ]));
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.max_derivation_depth = 20;
    config.allow_remnant_movement = true;
    parser.set_config(config);
    
    parser
}

// Setup a head-final language parser (Japanese-like)
fn setup_head_final_parser() -> MinimalistParser {
    let mut parser = MinimalistParser::new();
    
    // Nouns and proper names
    parser.add_to_lexicon("John", LexicalItem::new("John", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("Mary", LexicalItem::new("Mary", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("cat", LexicalItem::new("cat", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("book", LexicalItem::new("book", vec![
        Feature::categorial("D"),
    ]));
    
    // Head-final verbs (object-subject-verb order)
    parser.add_to_lexicon("saw", LexicalItem::new("saw", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject (appears before verb)
        Feature::selector("D"),  // Object (appears before subject)
    ]));
    
    parser.add_to_lexicon("slept", LexicalItem::new("slept", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject (appears before verb)
    ]));
    
    parser.add_to_lexicon("gave", LexicalItem::new("gave", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject (appears before verb)
        Feature::selector("D"),  // Indirect object (appears before subject)
        Feature::selector("D"),  // Direct object (appears before indirect object)
    ]));
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.max_derivation_depth = 20;
    parser.set_config(config);
    
    parser
}

// Setup a parser with agreement features
fn setup_agreement_parser() -> MinimalistParser {
    let mut parser = MinimalistParser::new();
    
    // Singular determiners
    let mut sg_agr = FeatureStructure::new();
    sg_agr.add("num", "sg".into());
    
    parser.add_to_lexicon("the", LexicalItem::with_agreement("the", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
    ], sg_agr.clone()));
    
    parser.add_to_lexicon("a", LexicalItem::with_agreement("a", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
    ], sg_agr.clone()));
    
    // Plural determiners
    let mut pl_agr = FeatureStructure::new();
    pl_agr.add("num", "pl".into());
    
    parser.add_to_lexicon("the", LexicalItem::with_agreement("the", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
    ], pl_agr.clone()));
    
    parser.add_to_lexicon("some", LexicalItem::with_agreement("some", vec![
        Feature::categorial("D"),
        Feature::selector("N"),
    ], pl_agr.clone()));
    
    // Singular nouns
    parser.add_to_lexicon("cat", LexicalItem::with_agreement("cat", vec![
        Feature::categorial("N"),
    ], sg_agr.clone()));
    
    // Plural nouns
    parser.add_to_lexicon("cats", LexicalItem::with_agreement("cats", vec![
        Feature::categorial("N"),
    ], pl_agr.clone()));
    
    // Singular verbs
    parser.add_to_lexicon("sleeps", LexicalItem::with_agreement("sleeps", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject
        Feature::agreement("num", "sg"),  // Agreement with subject
    ], sg_agr.clone()));
    
    // Plural verbs
    parser.add_to_lexicon("sleep", LexicalItem::with_agreement("sleep", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Subject
        Feature::agreement("num", "pl"),  // Agreement with subject
    ], pl_agr.clone()));
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.max_derivation_depth = 20;
    parser.set_config(config);
    
    parser
}

// Setup a parser with phase-based constraints
fn setup_phase_parser() -> MinimalistParser {
    let mut parser = MinimalistParser::new();
    
    // Proper names
    parser.add_to_lexicon("John", LexicalItem::new("John", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("Mary", LexicalItem::new("Mary", vec![
        Feature::categorial("D"),
    ]));
    
    parser.add_to_lexicon("Bill", LexicalItem::new("Bill", vec![
        Feature::categorial("D"),
    ]));
    
    // Verbs
    parser.add_to_lexicon("saw", LexicalItem::new("saw", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    // Phase head v (light verb)
    parser.add_to_lexicon("v", LexicalItem::new("", vec![
        Feature::categorial("v"),
        Feature::selector("V"),
        Feature::phase("v"),  // v is a phase head
    ]));
    
    // T (tense)
    parser.add_to_lexicon("T", LexicalItem::new("", vec![
        Feature::categorial("T"),
        Feature::selector("v"),
    ]));
    
    // C (complementizer)
    parser.add_to_lexicon("that", LexicalItem::new("that", vec![
        Feature::categorial("C"),
        Feature::selector("T"),
        Feature::phase("C"),  // C is a phase head
    ]));
    
    // Attitude verb
    parser.add_to_lexicon("thinks", LexicalItem::new("thinks", vec![
        Feature::categorial("V"),
        Feature::selector("C"),  // Complement clause
        Feature::selector("D"),  // Subject
    ]));
    
    // Wh-words
    parser.add_to_lexicon("what", LexicalItem::new("what", vec![
        Feature::categorial("D"),
        Feature::licensee("wh"),  // For wh-movement
    ]));
    
    // Question C
    parser.add_to_lexicon("does", LexicalItem::new("does", vec![
        Feature::categorial("C"),
        Feature::licensor("wh"),  // Attracts wh-phrases
        Feature::selector("T"),
        Feature::phase("C"),  // C is a phase head
    ]));
    
    // Configure parser with phase constraints
    let mut config = ParserConfig::default();
    config.max_derivation_depth = 30;
    config.allow_remnant_movement = true;
    config.phase_config.enforce_pic = true;  // Enforce Phase Impenetrability Condition
    config.phase_config.immediate_transfer = true;
    parser.set_config(config);
    
    parser
}
