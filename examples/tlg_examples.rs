use nlparsers::common::Parser;
use nlparsers::tlg::{
    TLGParser, LogicalType, ParserConfig, StructuralProperty
};
use nlparsers::common::{FeatureStructure, FeatureValue};

fn main() {
    println!("=== TYPE-LOGICAL GRAMMAR PARSER ===");
    
    // English examples
    println!("\n--- ENGLISH EXAMPLES ---");
    let english_parser = setup_english_parser();
    
    let english_sentences = [
        "the cat sleeps",
        "a man sees a woman",
    ];
    
    for sentence in &english_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(proof) = english_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Proof tree:");
            println!("{}", proof);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Dutch examples (with crossed dependencies)
    println!("\n--- DUTCH EXAMPLES ---");
    let dutch_parser = setup_dutch_parser();
    
    let dutch_sentences = [
        "Jan Marie zag zwemmen",  // "Jan saw Marie swim" (crossed dependency)
    ];
    
    for sentence in &dutch_sentences {
        println!("\nParsing: {}", sentence);
        if let Some(proof) = dutch_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Proof tree:");
            println!("{}", proof);
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
        if let Some(proof) = japanese_parser.parse(sentence) {
            println!("✓ Valid parse found");
            println!("Proof tree:");
            println!("{}", proof);
        } else {
            println!("✗ No valid parse found");
        }
    }
    
    // Modal logic examples
    println!("\n--- MODAL LOGIC EXAMPLES ---");
    let modal_parser = setup_modal_logic_parser();
    
    let formulas = [
        "necessarily p",
        "p and q",
        "not possibly q",
    ];
    
    for formula in &formulas {
        println!("\nParsing: {}", formula);
        if let Some(proof) = modal_parser.parse(formula) {
            println!("✓ Valid parse found");
            println!("Proof tree:");
            println!("{}", proof);
        } else {
            println!("✗ No valid parse found");
        }
    }
}

// Setup an English Type-Logical Grammar parser
fn setup_english_parser() -> TLGParser {
    let mut parser = TLGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("s");
    parser.register_atomic_type("np");
    parser.register_atomic_type("n");
    
    // Register features
    parser.register_feature("num", &["sg", "pl"]);
    parser.register_feature("per", &["1", "2", "3"]);
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.use_features = true;
    parser.set_config(config);
    
    // Add lexical entries
    
    // Basic types
    let np = LogicalType::np();
    let s = LogicalType::s();
    let n = LogicalType::n();
    
    // Determiners: (np ← n)
    let det_type = LogicalType::left_impl(np.clone(), n.clone());
    parser.add_to_lexicon("the", det_type.clone());
    parser.add_to_lexicon("a", det_type.clone());
    
    // Nouns: n
    parser.add_to_lexicon("cat", n.clone());
    parser.add_to_lexicon("dog", n.clone());
    parser.add_to_lexicon("man", n.clone());
    parser.add_to_lexicon("woman", n.clone());
    
    // Intransitive verbs: (s ← np)
    let iv_type = LogicalType::left_impl(s.clone(), np.clone());
    parser.add_to_lexicon("sleeps", iv_type.clone());
    parser.add_to_lexicon("runs", iv_type.clone());
    
    // Transitive verbs: ((s ← np) ← np)
    let tv_type = LogicalType::left_impl(iv_type.clone(), np.clone());
    parser.add_to_lexicon("sees", tv_type.clone());
    parser.add_to_lexicon("chases", tv_type.clone());
    
    parser
}

// Setup a Dutch Type-Logical Grammar parser with cross-serial dependencies
fn setup_dutch_parser() -> TLGParser {
    let mut parser = TLGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("s");
    parser.register_atomic_type("np");
    parser.register_atomic_type("vp");
    parser.register_atomic_type("inf"); // Infinitive
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.use_product = true;
    config.use_displacement = true;
    parser.set_config(config);
    
    // Add lexical entries
    
    let s = LogicalType::s();
    let np = LogicalType::np();
    let inf = LogicalType::atomic("inf");
    
    // Nouns and proper names
    parser.add_to_lexicon("Jan", np.clone());
    parser.add_to_lexicon("Marie", np.clone());
    
    // Verbs: handle cross-serial dependencies
    
    // "zag" (saw) takes an NP and a VP[inf] argument
    // For cross-serial dependencies, use product types
    let perception_verb = LogicalType::left_impl(
        s.clone(),
        LogicalType::product(
            np.clone(),
            inf.clone()
        )
    );
    parser.add_to_lexicon("zag", perception_verb);
    
    // "zwemmen" (swim) is an infinitive verb taking an NP subject
    let inf_verb = LogicalType::left_impl(inf.clone(), np.clone());
    parser.add_to_lexicon("zwemmen", inf_verb);
    
    parser
}

// Setup a Japanese Type-Logical Grammar parser
fn setup_japanese_parser() -> TLGParser {
    let mut parser = TLGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("s");
    parser.register_atomic_type("np");
    parser.register_atomic_type("n");
    parser.register_atomic_type("pp");
    
    // Register features
    parser.register_feature("case", &["nom", "acc", "dat"]);
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.use_features = true;
    parser.set_config(config);
    
    // Add lexical entries
    
    let s = LogicalType::s();
    let n = LogicalType::n();
    
    // Case-marked noun phrases
    let mut nom_feat = FeatureStructure::new();
    nom_feat.add("case", FeatureValue::Atomic("nom".to_string()));
    let np_nom = LogicalType::atomic_with_features("np", &nom_feat);
    
    let mut acc_feat = FeatureStructure::new();
    acc_feat.add("case", FeatureValue::Atomic("acc".to_string()));
    let np_acc = LogicalType::atomic_with_features("np", &acc_feat);
    
    let mut dat_feat = FeatureStructure::new();
    dat_feat.add("case", FeatureValue::Atomic("dat".to_string()));
    let np_dat = LogicalType::atomic_with_features("np", &dat_feat);
    
    // Case markers (postpositions)
    parser.add_to_lexicon("が", LogicalType::left_impl(np_nom.clone(), n.clone())); // Nominative
    parser.add_to_lexicon("を", LogicalType::left_impl(np_acc.clone(), n.clone())); // Accusative
    parser.add_to_lexicon("に", LogicalType::left_impl(np_dat.clone(), n.clone())); // Dative
    
    // Nouns
    parser.add_to_lexicon("猫", n.clone()); // "cat"
    parser.add_to_lexicon("太郎", n.clone()); // "Taro"
    parser.add_to_lexicon("花子", n.clone()); // "Hanako"
    parser.add_to_lexicon("本", n.clone()); // "book"
    
    // Verbs (head-final)
    // Intransitive: NP-nom + V
    let iv_type = LogicalType::left_impl(s.clone(), np_nom.clone());
    parser.add_to_lexicon("眠る", iv_type); // "sleep"
    
    // Ditransitive: NP-nom + NP-dat + NP-acc + V
    let ditrans_type = LogicalType::left_impl(
        LogicalType::left_impl(
            LogicalType::left_impl(s.clone(), np_nom.clone()),
            np_dat.clone()
        ),
        np_acc.clone()
    );
    parser.add_to_lexicon("渡す", ditrans_type); // "give"
    
    parser
}

// Setup a modal logic parser
fn setup_modal_logic_parser() -> TLGParser {
    let mut parser = TLGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("s");  // Sentence/proposition
    parser.register_atomic_type("np"); // Noun phrase
    parser.register_atomic_type("n");  // Noun
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.use_modalities = true;
    config.logic_variant = "NL(◇)".to_string();
    parser.set_config(config);
    
    // Add modalities
    parser.register_modality(1, vec![StructuralProperty::Associativity]);
    
    // Add lexical entries
    
    let s = LogicalType::s();
    
    // Propositional variables
    parser.add_to_lexicon("p", s.clone());
    parser.add_to_lexicon("q", s.clone());
    parser.add_to_lexicon("r", s.clone());
    
    // Logical operators
    
    // Conjunction: s ← s ← s
    let conj_type = LogicalType::left_impl(
        LogicalType::left_impl(s.clone(), s.clone()),
        s.clone()
    );
    parser.add_to_lexicon("and", conj_type);
    
    // Disjunction: s ← s ← s
    let disj_type = LogicalType::left_impl(
        LogicalType::left_impl(s.clone(), s.clone()),
        s.clone()
    );
    parser.add_to_lexicon("or", disj_type);
    
    // Negation: s ← s
    let neg_type = LogicalType::left_impl(s.clone(), s.clone());
    parser.add_to_lexicon("not", neg_type);
    
    // Modal operators
    
    // Necessity: s ← s with box modality
    let nec_type = LogicalType::left_impl(s.clone(), LogicalType::boxed(s.clone()));
    parser.add_to_lexicon("necessarily", nec_type);
    
    // Possibility: s ← s with diamond modality
    let poss_type = LogicalType::left_impl(s.clone(), LogicalType::diamond(s.clone()));
    parser.add_to_lexicon("possibly", poss_type);
    
    parser
}