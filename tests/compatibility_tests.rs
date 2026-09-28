use stranger_strings::{StrangerStrings, AnalysisOptions};

#[test]
fn test_score_compatibility_with_typescript() {
    // Test cases that should match the TypeScript implementation exactly
    let test_strings = vec![
        "hello",
        "world", 
        "function",
        "initialize",
        "process",
        "file_inherit",
        "total %qu",
        "Error: %s",
        "main()",
        "sizeof",
        ".CRT$XIC",
        "Ta&@",
        "xZ#@$%",
        "!@#$%^&*",
        "}{][++",
        "ab",
        "a",
        "",
        "123",
        "XML",
    ];

    // Expected results based on TypeScript output shown in verbose test
    let expected_results = vec![
        ("hello", true, -2.925, -3.260),
        ("world", true, -3.209, -3.260),
        ("function", true, -2.675, -4.230),
        ("initialize", true, -2.970, -4.550),
        ("process", true, -2.734, -3.840),
        ("file_inherit", true, -3.413, -4.880),
        ("total %qu", true, -3.946, -4.490),
        ("Error: %s", true, -3.172, -4.490),
        ("main()", false, -3.975, -3.520),
        ("sizeof", true, -3.349, -3.520),
        (".CRT$XIC", false, -4.873, -4.230),
        ("Ta&@", false, -4.386, -2.710),
        ("xZ#@$%", false, -5.852, -3.520),
        ("!@#$%^&*", false, -5.994, -4.230),
        ("}{][++", false, -5.588, -3.520),
        ("ab", false, -20.000, 10.000),
        ("a", false, -20.000, 10.000),
        ("", false, -20.000, 10.000),
        ("123", false, -3.812, 10.000),
        ("XML", false, -3.139, 10.000),
    ];

    // Test with the real model if available
    let mut analyzer = StrangerStrings::new();
    let load_result = analyzer.load_model(&AnalysisOptions {
        model_path: Some("./StringModel.sng".to_string()),
        ..Default::default()
    });

    if load_result.is_ok() {
        // Verify model is configured correctly  
        let (model_type, is_lowercase) = analyzer.get_model_info().unwrap();
        assert_eq!(model_type, "lowercase");
        assert!(is_lowercase);

        // Test each string and compare with expected results
        for (i, test_string) in test_strings.iter().enumerate() {
            let result = analyzer.analyze_string(test_string).unwrap();
            let (expected_string, expected_valid, expected_score, expected_threshold) = &expected_results[i];
            
            assert_eq!(result.original_string, *expected_string);
            assert_eq!(result.is_valid, *expected_valid, 
                      "Validity mismatch for '{}': expected {}, got {}", 
                      test_string, expected_valid, result.is_valid);
            
            // Allow small floating point differences (within 0.001)
            assert!((result.score - expected_score).abs() < 0.001, 
                   "Score mismatch for '{}': expected {:.3}, got {:.3}", 
                   test_string, expected_score, result.score);
            assert!((result.threshold - expected_threshold).abs() < 0.001,
                   "Threshold mismatch for '{}': expected {:.3}, got {:.3}", 
                   test_string, expected_threshold, result.threshold);
        }
        
        println!("✅ All {} test strings match TypeScript output exactly", test_strings.len());
    } else {
        println!("⚠️  StringModel.sng not found, skipping compatibility test");
    }
}

#[test]
fn test_binary_extraction_functionality() {
    // Test binary string extraction with known patterns
    let test_binary = b"Hello\x00World\x00\xFF\xFE\x00Function\x00Test\x01\x02String";
    
    let mut analyzer = StrangerStrings::new();
    
    // Test with a minimal model for functionality
    let model_content = r#"# Model Type: lowercase
[^]	h	e	100
h	e	l	200
e	l	l	150
l	l	o	180
l	o	[$]	90
[^]	w	o	50
w	o	r	80
o	r	l	70
r	l	d	60
l	d	[$]	40
"#;

    analyzer.load_model(&AnalysisOptions {
        model_content: Some(model_content.to_string()),
        ..Default::default()
    }).unwrap();

    // Extract strings from binary
    let extracted = analyzer.extract_strings_from_binary(test_binary, 4);
    
    // Should extract: "Hello", "World", "Function", "Test", "String"
    assert!(extracted.len() >= 4);
    
    let extracted_strings: Vec<&str> = extracted.iter().map(|bs| bs.string.as_str()).collect();
    assert!(extracted_strings.contains(&"Hello"));
    assert!(extracted_strings.contains(&"World"));
    assert!(extracted_strings.contains(&"Function"));
    assert!(extracted_strings.contains(&"String"));
    
    // Verify offsets are tracked correctly
    let hello_entry = extracted.iter().find(|bs| bs.string == "Hello").unwrap();
    assert_eq!(hello_entry.offset, 0);
    
    let world_entry = extracted.iter().find(|bs| bs.string == "World").unwrap();
    assert_eq!(world_entry.offset, 6);
}

#[test]
fn test_edge_cases_and_robustness() {
    let mut analyzer = StrangerStrings::new();
    
    // Test with minimal model
    let model_content = r#"# Model Type: lowercase
[^]	t	e	10
t	e	s	20
e	s	t	15
s	t	[$]	5
"#;

    analyzer.load_model(&AnalysisOptions {
        model_content: Some(model_content.to_string()),
        ..Default::default()
    }).unwrap();

    // Test empty strings
    let result = analyzer.analyze_string("").unwrap();
    assert!(!result.is_valid);
    assert_eq!(result.score, -20.0);

    // Test very short strings
    let result = analyzer.analyze_string("a").unwrap();
    assert!(!result.is_valid);
    assert_eq!(result.score, -20.0);

    // Test string with non-ASCII characters
    let result = analyzer.analyze_string("tëst").unwrap();
    assert_eq!(result.normalized_string, "t st"); // ë replaced with space

    // Test case conversion for lowercase model
    let result_lower = analyzer.analyze_string("test").unwrap();
    let result_upper = analyzer.analyze_string("TEST").unwrap();
    assert_eq!(result_lower.normalized_string, result_upper.normalized_string);
    assert_eq!(result_lower.score, result_upper.score);

    // Test batch processing
    let batch_results = analyzer.analyze_strings(&[
        "test".to_string(),
        "TEST".to_string(), 
        "invalid".to_string()
    ]).unwrap();
    assert_eq!(batch_results.len(), 3);
    
    // Test valid string extraction
    let valid_only = analyzer.extract_valid_strings(&[
        "test".to_string(),
        "xyz".to_string(),
        "random".to_string()
    ]).unwrap();
    
    // With our minimal test model, may or may not have valid strings
    // Just verify the function works without crashing
    println!("Found {} valid strings from batch", valid_only.len());
}