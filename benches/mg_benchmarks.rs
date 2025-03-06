use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nlparsers::mg::{Feature, LexicalItem, MinimalistParser, ParserConfig};
use nlparsers::common::Parser;

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
    parser.add_to_lexicon("sees", LexicalItem::new("sees", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    parser.add_to_lexicon("chases", LexicalItem::new("chases", vec![
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
    
    // Nouns
    parser.add_to_lexicon("book", LexicalItem::new("book", vec![
        Feature::categorial("N"),
    ]));
    
    parser.add_to_lexicon("person", LexicalItem::new("person", vec![
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
    parser.add_to_lexicon("read", LexicalItem::new("read", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    parser.add_to_lexicon("saw", LexicalItem::new("saw", vec![
        Feature::categorial("V"),
        Feature::selector("D"),  // Object
        Feature::selector("D"),  // Subject
    ]));
    
    // Tense
    parser.add_to_lexicon("did", LexicalItem::new("did", vec![
        Feature::categorial("T"),
        Feature::selector("V"),
        Feature::selector("D"),  // Subject
    ]));
    
    // Question C
    parser.add_to_lexicon("", LexicalItem::new("", vec![
        Feature::categorial("C"),
        Feature::licensor("wh"),  // Attracts wh-phrases
        Feature::selector("T"),
    ]));
    
    // Configure parser
    let mut config = ParserConfig::default();
    config.max_derivation_depth = 20;
    parser.set_config(config);
    
    parser
}

fn bench_mg_parsing(c: &mut Criterion) {
    let english_parser = setup_english_parser();
    let wh_parser = setup_wh_parser();
    
    let simple_sentence = "the cat sleeps";
    let complex_sentence = "the dog chases the cat";
    let wh_question = "what did John read";
    
    let mut group = c.benchmark_group("MG Parsing");
    
    group.bench_function("simple sentence", |b| {
        b.iter(|| english_parser.parse(black_box(simple_sentence)))
    });
    
    group.bench_function("complex sentence", |b| {
        b.iter(|| english_parser.parse(black_box(complex_sentence)))
    });
    
    group.bench_function("wh-movement", |b| {
        b.iter(|| wh_parser.parse(black_box(wh_question)))
    });
    
    group.finish();
}

criterion_group!(benches, bench_mg_parsing);
criterion_main!(benches);
