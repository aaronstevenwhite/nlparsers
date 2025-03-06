use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nlparsers::ccg::{CCGParser, CCGCategory, CCGParserConfig};
use nlparsers::common::Parser;

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
    
    // Intransitive verbs: S\NP
    let iv_type = CCGCategory::backward(s.clone(), np.clone());
    parser.add_to_lexicon("sleeps", iv_type.clone());
    parser.add_to_lexicon("runs", iv_type.clone());
    
    // Transitive verbs: (S\NP)/NP
    let tv_type = CCGCategory::forward(iv_type.clone(), np.clone());
    parser.add_to_lexicon("sees", tv_type.clone());
    parser.add_to_lexicon("chases", tv_type.clone());
    
    // Configure parser
    let mut config = CCGParserConfig::default();
    config.max_composition_order = 2;
    parser.set_config(config);
    
    parser
}

fn bench_ccg_parsing(c: &mut Criterion) {
    let parser = setup_english_parser();
    
    let simple_sentence = "the cat sleeps";
    let complex_sentence = "the dog chases the cat";
    
    let mut group = c.benchmark_group("CCG Parsing");
    
    group.bench_function("simple sentence", |b| {
        b.iter(|| parser.parse(black_box(simple_sentence)))
    });
    
    group.bench_function("complex sentence", |b| {
        b.iter(|| parser.parse(black_box(complex_sentence)))
    });
    
    group.finish();
}

criterion_group!(benches, bench_ccg_parsing);
criterion_main!(benches);
