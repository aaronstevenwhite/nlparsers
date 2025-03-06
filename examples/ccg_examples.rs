use nlparsers::ccg::{
    CCGParser, CCGCategory, CCGParserConfig
};
use nlparsers::common::Parser;

fn main() {
    println!("=== COMBINATORY CATEGORIAL GRAMMAR PARSER ===");
    
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
    
    // Dutch examples (with cross-serial dependencies)
    println!("\n--- DUTCH EXAMPLES ---");
    let dutch_parser = setup_dutch_parser();
    
    let dutch_sentences = [
        "Jan Marie ziet zwemmen",  // "Jan sees Marie swim" (crossed dependency)
    ];
    
    for sentence in &dutch_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = dutch_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Japanese examples (head-final language)
    println!("\n--- JAPANESE EXAMPLES ---");
    let japanese_parser = setup_japanese_parser();
    
    let japanese_sentences = [
        "猫が 眠る",            // "cat sleeps"
        "太郎が 花子に 本を 渡す", // "Taro gives a book to Hanako"
    ];
    
    for sentence in &japanese_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(parse_tree) = japanese_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Parse tree:");
            println!("{}", parse_tree);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Morphosyntactic features examples
    println!("\n--- MORPHOSYNTACTIC AGREEMENT EXAMPLES ---");
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
}

// Setup a basic English CCG parser
fn setup_english_parser() -> CCGParser {
    let mut parser = CCGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("S");
    parser.register_atomic_type("NP");
    parser.register_atomic_type("N");
    
    // Basic categories
    let s = CCGCategory::s();
    let np = CCGCategory::np();
    let n = CCGCategory::n();
    
    // Determiners: NP/N
    let det_type = CCGCategory::forward(np.clone(), n.clone());
    parser.add_to_lexicon("the", det_type.clone());
    parser.add_to_lexicon("a", det_type.clone());
    
    // Nouns: N
    parser.add_to_lexicon("cat", n.clone());
    parser.add_to_lexicon("dog", n.clone());
    parser.add_to_lexicon("man", n.clone());
    parser.add_to_lexicon("woman", n.clone());
    parser.add_to_lexicon("Mary", np.clone());
    parser.add_to_lexicon("John", np.clone());
    
    // Intransitive verbs: S\NP
    let iv_type = CCGCategory::backward(s.clone(), np.clone());
    parser.add_to_lexicon("sleeps", iv_type.clone());
    parser.add_to_lexicon("runs", iv_type.clone());
    
    // Transitive verbs: (S\NP)/NP
    let tv_type = CCGCategory::forward(iv_type.clone(), np.clone());
    parser.add_to_lexicon("sees", tv_type.clone());
    parser.add_to_lexicon("chases", tv_type.clone());
    parser.add_to_lexicon("loves", tv_type.clone());
    
    // Auxiliary verbs: (S\NP)/(S\NP)
    let aux_type = CCGCategory::forward(iv_type.clone(), iv_type.clone());
    parser.add_to_lexicon("will", aux_type.clone());
    
    // Relative pronouns: (N\N)/(S\NP)
    let rel_type = CCGCategory::forward(
        CCGCategory::backward(n.clone(), n.clone()),
        iv_type.clone()
    );
    parser.add_to_lexicon("who", rel_type.clone());
    parser.add_to_lexicon("that", rel_type.clone());
    
    // Configure parser for composition
    let mut config = CCGParserConfig::default();
    config.max_composition_order = 2;
    config.enable_type_raising = true;
    parser.set_config(config);
    
    parser
}

// Setup a Dutch CCG parser with cross-serial dependencies
fn setup_dutch_parser() -> CCGParser {
    let mut parser = CCGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("S");
    parser.register_atomic_type("NP");
    parser.register_atomic_type("VP");
    parser.register_atomic_type("INF");
    
    // Basic categories
    let s = CCGCategory::s();
    let np = CCGCategory::np();
    let inf = CCGCategory::atomic("INF");
    
    // Proper names: NP
    parser.add_to_lexicon("Jan", np.clone());
    parser.add_to_lexicon("Marie", np.clone());
    
    // Perception verb "ziet" (sees): (S\NP)/VP
    let vp = CCGCategory::backward(inf.clone(), np.clone());
    let perception_verb = CCGCategory::forward(
        CCGCategory::backward(s.clone(), np.clone()),
        vp.clone()
    );
    parser.add_to_lexicon("ziet", perception_verb);
    
    // Infinitive verb "zwemmen" (swim): VP\NP
    let inf_verb = CCGCategory::backward(inf.clone(), np.clone());
    parser.add_to_lexicon("zwemmen", inf_verb);
    
    // Configure parser for composition
    let mut config = CCGParserConfig::default();
    config.max_composition_order = 3;  // Need higher-order composition for cross-serial dependencies
    config.enable_type_raising = true;
    parser.set_config(config);
    
    parser
}

// Setup a Japanese CCG parser (head-final language)
fn setup_japanese_parser() -> CCGParser {
    let mut parser = CCGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("S");
    parser.register_atomic_type("NP");
    parser.register_atomic_type("N");
    
    // Basic categories
    let s = CCGCategory::s();
    let np = CCGCategory::np();
    let n = CCGCategory::n();
    
    // Case markers (postpositions)
    // Nominative: NP\N
    parser.add_to_lexicon("が", CCGCategory::backward(np.clone(), n.clone()));
    // Accusative: NP\N
    parser.add_to_lexicon("を", CCGCategory::backward(np.clone(), n.clone()));
    // Dative: NP\N
    parser.add_to_lexicon("に", CCGCategory::backward(np.clone(), n.clone()));
    
    // Nouns: N
    parser.add_to_lexicon("猫", n.clone());  // "cat"
    parser.add_to_lexicon("太郎", n.clone()); // "Taro"
    parser.add_to_lexicon("花子", n.clone()); // "Hanako"
    parser.add_to_lexicon("本", n.clone());  // "book"
    
    // Intransitive verbs: S\NP
    let iv_type = CCGCategory::backward(s.clone(), np.clone());
    parser.add_to_lexicon("眠る", iv_type.clone());  // "sleep"
    
    // Ditransitive verb: ((S\NP)\NP)\NP
    //let tv_type = CCGCategory::backward(iv_type.clone(), np.clone());
    let ditrans_type = CCGCategory::backward(
        CCGCategory::backward(iv_type.clone(), np.clone()),
        np.clone()
    );
    parser.add_to_lexicon("渡す", ditrans_type);  // "give"
    
    // Configure parser
    let mut config = CCGParserConfig::default();
    config.max_composition_order = 2;
    parser.set_config(config);
    
    parser
}

// Setup a CCG parser with morphosyntactic agreement
fn setup_agreement_parser() -> CCGParser {
    let mut parser = CCGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("S");
    parser.register_atomic_type("NP");
    parser.register_atomic_type("N");
    
    // Register features
    parser.register_feature_dimension("num", &["sg", "pl"]);
    
    // Enable morphosyntactic features
    let mut config = CCGParserConfig::default();
    config.use_morphosyntax = true;
    config.enforce_feature_unification = true;
    parser.set_config(config);
    
    // Create categories with features
    let s = CCGCategory::s();
    
    // Singular and plural nouns
    let n_sg = CCGCategory::n_with_number("sg");
    let n_pl = CCGCategory::n_with_number("pl");
    
    // Singular and plural NPs
    let np_sg = CCGCategory::np_with_features("nom", "sg");
    let np_pl = CCGCategory::np_with_features("nom", "pl");
    
    // Determiners with number agreement
    parser.add_to_lexicon("the", CCGCategory::forward(np_sg.clone(), n_sg.clone()));
    parser.add_to_lexicon("the", CCGCategory::forward(np_pl.clone(), n_pl.clone()));
    parser.add_to_lexicon("a", CCGCategory::forward(np_sg.clone(), n_sg.clone()));
    parser.add_to_lexicon("some", CCGCategory::forward(np_pl.clone(), n_pl.clone()));
    
    // Nouns with number
    parser.add_to_lexicon("cat", n_sg.clone());
    parser.add_to_lexicon("cats", n_pl.clone());
    
    // Verbs with subject agreement
    parser.add_to_lexicon("sleeps", CCGCategory::backward(s.clone(), np_sg.clone()));
    parser.add_to_lexicon("sleep", CCGCategory::backward(s.clone(), np_pl.clone()));
    
    parser
}
