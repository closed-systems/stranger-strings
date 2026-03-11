use stranger_strings_rs::language::chinese::ChineseAnalyzer;
use stranger_strings_rs::language::cyrillic::CyrillicAnalyzer;
use stranger_strings_rs::language::arabic::ArabicAnalyzer;

#[test]
fn test_debug_chinese_scoring() {
    let score1 = ChineseAnalyzer::score_chinese_text("你好");
    let score2 = ChineseAnalyzer::score_chinese_text("hello");
    let score3 = ChineseAnalyzer::score_chinese_text("你好世界");
    
    // Get detailed stats
    let stats1 = ChineseAnalyzer::get_chinese_stats("你好");
    let stats3 = ChineseAnalyzer::get_chinese_stats("你好世界");
    
    println!("=== Chinese Scoring Debug ===");
    println!("你好: {:.3} (han: {}, total: {}, unique: {}, common: {})", 
             score1, stats1.han_characters, stats1.total_characters, stats1.unique_characters, stats1.common_characters);
    println!("hello: {:.3}", score2);
    println!("你好世界: {:.3} (han: {}, total: {}, unique: {}, common: {})", 
             score3, stats3.han_characters, stats3.total_characters, stats3.unique_characters, stats3.common_characters);
    println!("Expected: score3 ({:.3}) > score1 ({:.3}): {}", score3, score1, score3 > score1);
    
    // Chinese should score better than English
    assert!(score1 > score2, "Chinese should score better than English: {} > {}", score1, score2);
    
    // Non-Chinese should get very low score
    assert!(score2 < -10.0, "Non-Chinese should get very low score: {}", score2);
}

#[test]
fn test_debug_cyrillic_scoring() {
    let score1 = CyrillicAnalyzer::score_cyrillic_text("привет");
    let score2 = CyrillicAnalyzer::score_cyrillic_text("hello");
    let score3 = CyrillicAnalyzer::score_cyrillic_text("русский язык");
    
    // Get detailed stats
    let stats1 = CyrillicAnalyzer::get_cyrillic_stats("привет");
    let stats3 = CyrillicAnalyzer::get_cyrillic_stats("русский язык");
    
    println!("=== Cyrillic Scoring Debug ===");
    println!("привет: {:.3} (cyr: {}, total: {}, unique: {}, common: {})", 
             score1, stats1.cyrillic_characters, stats1.total_characters, stats1.unique_characters, stats1.common_characters);
    println!("hello: {:.3}", score2);
    println!("русский язык: {:.3} (cyr: {}, total: {}, unique: {}, common: {})", 
             score3, stats3.cyrillic_characters, stats3.total_characters, stats3.unique_characters, stats3.common_characters);
    println!("Expected: score3 ({:.3}) > score1 ({:.3}): {}", score3, score1, score3 > score1);
    
    // Cyrillic should score better than English
    assert!(score1 > score2, "Cyrillic should score better than English: {} > {}", score1, score2);
    
    // Non-Cyrillic should get very 2low score
    assert!(score2 < -10.0, "Non-Cyrillic should get very low score: {}", score2);
    
    // Test pattern scoring issue
    let pattern_score1 = CyrillicAnalyzer::score_cyrillic_text("это");
    let pattern_score2 = CyrillicAnalyzer::score_cyrillic_text("бвгд");
    println!("Pattern test: 'это': {:.3}, 'бвгд': {:.3}, expected: score1 > score2", pattern_score1, pattern_score2);
}

#[test]
fn test_debug_arabic_scoring() {
    let score1 = ArabicAnalyzer::score_arabic_text("مرحبا");
    let score2 = ArabicAnalyzer::score_arabic_text("hello");
    let score3 = ArabicAnalyzer::score_arabic_text("مرحبا بالعالم");
    
    println!("=== Arabic Scoring Debug ===");
    println!("مرحبا: {:.3}", score1);
    println!("hello: {:.3}", score2);
    println!("مرحبا بالعالم: {:.3}", score3);
    println!("Expected: score3 ({:.3}) > score1 ({:.3}): {}", score3, score1, score3 > score1);
    
    // Arabic should score better than English
    assert!(score1 > score2, "Arabic should score better than English: {} > {}", score1, score2);
    
    // Non-Arabic should get very low score
    assert!(score2 < -10.0, "Non-Arabic should get very low score: {}", score2);
    
    println!("Why score3 <= score1: The scoring algorithm may be penalizing longer text");
}