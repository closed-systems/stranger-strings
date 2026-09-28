use stranger_strings::language::arabic::ArabicAnalyzer;
use stranger_strings::language::chinese::ChineseAnalyzer;
use stranger_strings::language::cyrillic::CyrillicAnalyzer;
fn show_score_arabic(text: &str) {
    let score = ArabicAnalyzer::score_arabic_text(&text);
    println!("{}: {:.3}", text, score);
}
fn main() {
    println!("=== Chinese Scoring ===");
    let score1 = ChineseAnalyzer::score_chinese_text("你好");
    let score2 = ChineseAnalyzer::score_chinese_text("hello");
    let score3 = ChineseAnalyzer::score_chinese_text("你好世界");
    println!("你好: {:.3}", score1);
    println!("hello: {:.3}", score2);
    println!("你好世界: {:.3}", score3);

    println!("\n=== Cyrillic Scoring ===");
    let c_score1 = CyrillicAnalyzer::score_cyrillic_text("привет");
    let c_score2 = CyrillicAnalyzer::score_cyrillic_text("hello");
    let c_score3 = CyrillicAnalyzer::score_cyrillic_text("русский язык");
    let c_score4 = CyrillicAnalyzer::score_cyrillic_text(
        "//TODO: заставить http-соединение работать правильно",
    );
    println!("привет: {:.3}", c_score1);
    println!("hello: {:.3}", c_score2);
    println!("русский язык: {:.3}", c_score3);
    println!(
        "//TODO: заставить http-соединение работать правильно: {:.3}",
        c_score4
    );
    println!("\n=== Arabic Scoring ===");
    show_score_arabic("مرحبا");
    show_score_arabic("hello");
    show_score_arabic("مرحبا بالعالم");
    show_score_arabic("Note: لا تقم بإزالة هذا الكود!!");
    show_score_arabic("خارطة تُظهرُ أبرز المعارك التي وقعت أثناء حُرُوب الرَّدة بين المُسلمين والقبائل العربيَّة المُرتدَّة عن الإسلام ومعهم مُدعي النُبوَّة.");
    // harakat string - has diacritic vowel markers for children, foreigners and places pronounciation is important (the Koran)
    show_score_arabic("خارطة تظهر أبرز المعارك التي وقعت أثناء حروب الردة بين المسلمين والقبائل العربية المرتدة عن الإسلام ومعهم مدعي النبوة.");
}
