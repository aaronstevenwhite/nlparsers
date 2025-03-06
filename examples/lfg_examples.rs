use nlparsers::lfg::{
    LFGParser, Category, FConstraint, CNode, Rule
};

fn main() {
    println!("=== LEXICAL-FUNCTIONAL GRAMMAR PARSER ===");
    
    // Create an LFG parser with English grammar
    let mut parser = LFGParser::new();
    parser.setup_english_grammar();
    
    // Enable constraint solving
    parser.config.enforce_well_formedness = true;
    parser.config.debug = true;
    
    // Display some information about the grammar
    println!("\nGrammar information:");
    println!("Number of rules: {}", parser.rules.len());
    println!("Lexicon entries: {}", parser.lexicon.get_words().len());
    
    println!("\nRules:");
    for (i, rule) in parser.rules.iter().enumerate() {
        println!("  {}: {}", i + 1, rule);
    }
    
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
            Some(parse_tree) => {
                println!("✓ Valid parse found");
                
                // Print C-structure
                println!("C-structure:");
                println!("{}", parse_tree);
                
                // Print F-structure of the root
                if let Some(f_structure) = &parse_tree.f_structure {
                    println!("Root F-structure:");
                    println!("{}", f_structure);
                }
                
                // Print subject's F-structure if present
                if let Some(f_structure) = &parse_tree.f_structure {
                    if let Some(subj_value) = f_structure.get("SUBJ") {
                        match subj_value {
                            rust_grammar_formalisms::lfg::FValue::Structure(fs) => {
                                println!("Subject F-structure:");
                                println!("{}", fs);
                            },
                            _ => {},
                        }
                    }
                }
            },
            None => {
                println!("✗ No valid parse found");
            }
        }
    }
    
    // Demonstrate adding a custom rule
    println!("\n=== Adding a custom rule ===");
    let s = Category::s();
    let np = Category::np();
    let pp = Category::new("PP");
    let vp = Category::vp();
    
    // Create S -> NP VP PP rule
    let mut s_with_pp_rule = Rule::new(s, vec![np, vp, pp]);
    s_with_pp_rule.annotate(0, vec![FConstraint::Equation("↑SUBJ".to_string(), "↓".to_string())]);
    s_with_pp_rule.annotate(1, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
    s_with_pp_rule.annotate(2, vec![FConstraint::Equation("↑ADJUNCT".to_string(), "↓".to_string())]);
    s_with_pp_rule.name = Some("S -> NP VP PP".to_string());
    
    println!("Custom rule: {}", s_with_pp_rule);
    
    // Add the rule to the parser
    parser.add_rule(s_with_pp_rule);
    
    // Add a PP rule
    let p = Category::new("P");
    let mut pp_rule = Rule::new(pp.clone(), vec![p, np.clone()]);
    pp_rule.annotate(0, vec![FConstraint::Equation("↑".to_string(), "↓".to_string())]);
    pp_rule.annotate(1, vec![FConstraint::Equation("↑OBJ".to_string(), "↓".to_string())]);
    pp_rule.name = Some("PP -> P NP".to_string());
    
    parser.add_rule(pp_rule);
    
    // Add lexical entries for prepositions
    parser.add_to_lexicon("in", Category::new("P"));
    parser.add_to_lexicon("with", Category::new("P"));
    
    // Test a sentence with the new rule
    let complex_sentence = "the cat sleeps in the house";
    println!("\nSentence with PP: {}", complex_sentence);
    
    match parser.parse(complex_sentence) {
        Some(parse_tree) => {
            println!("✓ Valid parse found");
            println!("C-structure:");
            println!("{}", parse_tree);
            
            if let Some(f_structure) = &parse_tree.f_structure {
                println!("Root F-structure:");
                println!("{}", f_structure);
            }
        },
        None => {
            println!("✗ No valid parse found");
        }
    }
}