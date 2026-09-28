use stranger_strings::{AnalysisOptions, StrangerStrings};

#[test]
fn test_real_model_loading() {
    let mut analyzer = StrangerStrings::new();

    // Test loading the actual model file
    let result = analyzer.load_model(&AnalysisOptions {
        model_path: Some("./StringModel.sng".to_string()),
        ..Default::default()
    });

    if result.is_ok() {
        // Model loaded successfully, verify it's configured correctly
        let (model_type, is_lowercase) = analyzer.get_model_info().unwrap();
        assert_eq!(model_type, "lowercase");
        assert!(is_lowercase);

        // Test some basic string analysis
        let result = analyzer.analyze_string("hello").unwrap();
        assert!(!result.original_string.is_empty());
        assert!(result.threshold < 0.0); // Should have a reasonable threshold

        let result = analyzer.analyze_string("test").unwrap();
        assert_eq!(result.original_string, "test");

        // Test that random strings get poor scores
        let result = analyzer.analyze_string("xqzfkj").unwrap();
        assert!(!result.is_valid); // Random string should be invalid
    } else {
        // Model file not found - skip this test
        println!("StringModel.sng not found, skipping real model test");
    }
}

#[test]
fn test_basic_analysis_workflow() {
    let mut analyzer = StrangerStrings::new();
    
    // Test loading the real default model
    let result = analyzer.load_model(&AnalysisOptions {
        model_path: Some("./StringModel.sng".to_string()),
        ..Default::default()
    });

    if result.is_err() {
        println!("StringModel.sng not found, skipping basic analysis workflow test");
        return;
    }

    // Verify model is loaded correctly
    let (model_type, is_lowercase) = analyzer.get_model_info().unwrap();
    assert_eq!(model_type, "lowercase");
    assert!(is_lowercase);

    // Test known good strings that should be valid with the real model
    let test_strings = vec![
        "hello".to_string(),
        "world".to_string(), 
        "function".to_string(),
        "initialize".to_string(),
        "process".to_string(),
    ];

    // Test individual string analysis
    for test_str in &test_strings {
        let result = analyzer.analyze_string(test_str).unwrap();
        assert_eq!(result.original_string, *test_str);
        assert!(!result.normalized_string.is_empty());
        // With the real model, these common English words should have reasonable scores
        assert!(result.score > -10.0);
    }

    // Test that strings get converted to lowercase for scoring
    let result_upper = analyzer.analyze_string("HELLO").unwrap();
    assert_eq!(result_upper.original_string, "HELLO");
    assert_eq!(result_upper.normalized_string, "hello");

    // Test batch analysis
    let results = analyzer.analyze_strings(&test_strings).unwrap();
    assert_eq!(results.len(), test_strings.len());
    
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.original_string, test_strings[i]);
    }

    // Test valid strings extraction - with real model these common words should be valid
    let valid_results = analyzer.extract_valid_strings(&test_strings).unwrap();
    
    // Should have some valid strings with the real model
    assert!(!valid_results.is_empty(), "Expected some valid results with real model");
}
