use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nlparsers::tlg::{TLGParser, LogicalType, ParserConfig, StructuralProperty};
use nlparsers::common::Parser;

// Setup an English Type-Logical Grammar parser (simplified from tlg_examples.rs)
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

// Setup a modal logic parser (simplified from tlg_examples.rs)
fn setup_modal_parser() -> TLGParser {
    let mut parser = TLGParser::new();
    
    // Register atomic types
    parser.register_atomic_type("s");
    
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
    
    // Modal operators
    let nec_type = LogicalType::left_impl(s.clone(), LogicalType::boxed(s.clone()));
    let poss_type = LogicalType::left_impl(s.clone(), LogicalType::diamond(s.clone()));
    parser.add_to_lexicon("necessarily", nec_type);
    parser.add_to_lexicon("possibly", poss_type);
    
    parser
}

fn bench_tlg_parsing(c: &mut Criterion) {
    let english_parser = setup_english_parser();
    let modal_parser = setup_modal_parser();
    
    let simple_sentence = "the cat sleeps";
    let complex_sentence = "a man sees a woman";
    let modal_formula = "necessarily p";
    
    let mut group = c.benchmark_group("TLG Parsing");
    
    group.bench_function("simple sentence", |b| {
        b.iter(|| english_parser.parse(black_box(simple_sentence)))
    });
    
    group.bench_function("complex sentence", |b| {
        b.iter(|| english_parser.parse(black_box(complex_sentence)))
    });
    
    group.bench_function("modal formula", |b| {
        b.iter(|| modal_parser.parse(black_box(modal_formula)))
    });
    
    group.finish();
}

criterion_group!(benches, bench_tlg_parsing);
criterion_main!(benches);
