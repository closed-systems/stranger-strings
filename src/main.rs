use clap::{Arg, ArgAction, Command};
use log::error;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use stranger_strings_rs::{
    get_threshold_for_length, AnalysisOptions, BinaryAnalysisOptions, StrangerError,
    StrangerStrings, StringAnalysisResult, SupportedEncoding, ScriptType, MAX_NG_THRESHOLD, NG_THRESHOLDS,
};

#[derive(Debug, Clone)]
struct CliOptions {
    model: String,
    verbose: bool,
    min_length: usize,
    output: Option<String>,
    format: String,
    unique: bool,
    sort: String,
    info: bool,
    test: bool,
    encodings: Vec<SupportedEncoding>,
    languages: Option<Vec<ScriptType>>,
    use_language_detection: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            model: default_model_path(),
            verbose: false,
            min_length: 4,
            output: None,
            format: "text".to_string(),
            unique: true,
            sort: "score".to_string(),
            info: false,
            test: false,
            encodings: vec![SupportedEncoding::Ascii], // Default to ASCII for backward compatibility
            languages: None,
            use_language_detection: false,
        }
    }
}

fn default_model_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|dir| dir.join("StringModel.sng")))
        .unwrap_or_else(|| PathBuf::from("./StringModel.sng"))
        .to_string_lossy()
        .into_owned()
}

fn main() {
    env_logger::init();

    if let Err(e) = run_main() {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_main() -> Result<(), StrangerError> {

    let matches = Command::new("stranger-strings")
        .about("Extract and analyze meaningful strings from binary files using trigram scoring")
        .version("0.1.0")
        .arg(Arg::new("input")
            .help("Input files to analyze, or \"-\" to read from stdin")
            .index(1)
            .num_args(1..))
        .arg(Arg::new("model")
            .short('m')
            .long("model")
            .value_name("PATH")
            .help("Path to .sng model file (default: StringModel.sng next to the executable)"))
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .help("Show detailed scoring information")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("min-length")
            .short('l')
            .long("min-length")
            .value_name("NUMBER")
            .help("Minimum string length for binary extraction")
            .default_value("4"))
        .arg(Arg::new("show-repeats")
            .long("show-repeats")
            .help("Show duplicate strings (default output is unique strings only)")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("sort")
            .short('s')
            .long("sort")
            .value_name("METHOD")
            .help("Sort results by: score (default), alpha (alphabetical), offset (file position)")
            .default_value("score")
            .value_parser(["score", "alpha", "offset"]))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("PATH")
            .help("Output file (default: stdout)"))
        .arg(Arg::new("format")
            .short('f')
            .long("format")
            .value_name("FORMAT")
            .help("Output format: text, json, csv")
            .default_value("text")
            .value_parser(["text", "json", "csv"]))
        .arg(Arg::new("info")
            .long("info")
            .help("Show model information and exit")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("test")
            .long("test")
            .help("Run with sample test strings")
            .action(ArgAction::SetTrue))
        .arg(Arg::new("encoding")
            .short('e')
            .long("encoding")
            .value_name("ENCODINGS")
            .help("Character encodings to use for string extraction (comma-separated). Available: ascii, utf8, utf16le, utf16be, latin1, latin9, all")
            .default_value("ascii"))
        .arg(Arg::new("language")
            .short('L')
            .long("language")
            .value_name("LANGUAGES")
            .help("Languages/scripts to detect and score (comma-separated). Available: latin, chinese, arabic, russian, all, auto"))
        .arg(Arg::new("auto-detect")
            .long("auto-detect")
            .help("Enable automatic language detection and use language-specific scoring")
            .action(ArgAction::SetTrue))
        .get_matches();

    let encodings = parse_encodings(matches.get_one::<String>("encoding").unwrap())?;
    let languages = if let Some(lang_str) = matches.get_one::<String>("language") {
        Some(parse_languages(lang_str)?)
    } else {
        None
    };
    let use_language_detection = matches.get_flag("auto-detect") || languages.is_some();
    
    let options = CliOptions {
        model: matches
            .get_one::<String>("model")
            .cloned()
            .unwrap_or_else(default_model_path),
        verbose: matches.get_flag("verbose"),
        min_length: matches
            .get_one::<String>("min-length")
            .unwrap()
            .parse()
            .unwrap_or(4),
        output: matches.get_one::<String>("output").cloned(),
        format: matches.get_one::<String>("format").unwrap().clone(),
        unique: !matches.get_flag("show-repeats"),
        sort: matches.get_one::<String>("sort").unwrap().clone(),
        info: matches.get_flag("info"),
        test: matches.get_flag("test"),
        encodings,
        languages,
        use_language_detection,
    };

    let result = if options.info {
        info_command(&options)
    } else if options.test {
        test_command(&options)
    } else {
        let inputs: Vec<String> = matches
            .get_many::<String>("input")
            .map(|vals| vals.cloned().collect())
            .unwrap_or_default();

        if inputs.is_empty() {
            eprintln!("Error: No input file specified. Use --help for usage information.");
            std::process::exit(1);
        }

        if options.output.is_some() && inputs.len() > 1 {
            return Err(StrangerError::InvalidInput(
                "--output can only be used with a single input file".to_string(),
            ));
        }

        let stdin_count = inputs.iter().filter(|input| input.as_str() == "-").count();
        if stdin_count > 1 {
            return Err(StrangerError::InvalidInput(
                "stdin input ('-') can only be specified once".to_string(),
            ));
        }

        for input in &inputs {
            analyze_command(input, &options)?;
        }

        Ok(())
    };

    result
}

fn parse_encodings(encoding_str: &str) -> Result<Vec<SupportedEncoding>, StrangerError> {
    if encoding_str.to_lowercase() == "all" {
        return Ok(SupportedEncoding::all());
    }
    
    let mut encodings = Vec::new();
    for part in encoding_str.split(',') {
        let trimmed = part.trim();
        let encoding = SupportedEncoding::from_str(trimmed)?;
        encodings.push(encoding);
    }
    
    if encodings.is_empty() {
        encodings.push(SupportedEncoding::Ascii);
    }
    
    Ok(encodings)
}

fn parse_languages(language_str: &str) -> Result<Vec<ScriptType>, StrangerError> {
    let lower_str = language_str.to_lowercase();
    
    if lower_str == "all" {
        return Ok(ScriptType::all());
    }
    
    if lower_str == "auto" {
        // Return empty vec to indicate auto-detection
        return Ok(vec![]);
    }
    
    let mut languages = Vec::new();
    for part in language_str.split(',') {
        let trimmed = part.trim();
        let language = ScriptType::from_str(trimmed)
            .ok_or_else(|| StrangerError::InvalidInput(format!("Unknown language/script: {}", trimmed)))?;
        languages.push(language);
    }
    
    if languages.is_empty() {
        languages.push(ScriptType::Latin);
    }
    
    Ok(languages)
}

fn analyze_command(input: &str, options: &CliOptions) -> Result<(), StrangerError> {
    let mut analyzer = StrangerStrings::new();

    // Load model or enable language detection
    if Path::new(&options.model).exists() {
        if options.verbose {
            eprintln!("Loading model: {}", options.model);
        }
        analyzer.load_model(&AnalysisOptions {
            model_path: Some(options.model.clone()),
            ..Default::default()
        })?;
    } else if options.use_language_detection {
        if options.verbose {
            eprintln!("Enabling language detection (no model file found)");
        }
        analyzer.enable_language_detection()?;
    } else {
        return Err(StrangerError::InvalidInput(format!(
            "Model file not found: {}",
            options.model
        )));
    }

    if options.verbose {
        if let Ok((model_type, is_lowercase)) = analyzer.get_model_info() {
            eprintln!("Model type: {}, Lowercase: {}", model_type, is_lowercase);
        } else if options.use_language_detection {
            eprintln!("Using language detection mode");
        }
    }

    let results: Vec<StringAnalysisResult>;
    let is_binary_file;

    if input == "-" {
        // Read from stdin
        if options.verbose {
            eprintln!("Reading from stdin...");
        }

        let mut input_data = String::new();
        io::stdin().read_to_string(&mut input_data)?;

        let candidate_strings: Vec<String> = input_data
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        results = analyzer.analyze_strings(&candidate_strings)?;
        is_binary_file = false;
    } else {
        // Analyze file
        if !Path::new(input).exists() {
            return Err(StrangerError::InvalidInput(format!(
                "File not found: {}",
                input
            )));
        }

        if options.verbose {
            eprintln!("Analyzing file: {}", input);
        }

        let buffer = fs::read(input)?;

        if options.verbose {
            let candidate_strings =
                analyzer.extract_strings_from_binary(&buffer, options.min_length);
            eprintln!(
                "Extracted {} candidate strings (min length: {})",
                candidate_strings.len(),
                options.min_length
            );
        }

        results = analyzer.analyze_binary_file(
            &buffer,
            &BinaryAnalysisOptions {
                min_length: Some(options.min_length),
                encodings: Some(options.encodings.clone()),
                target_languages: options.languages.clone(),
                use_language_scoring: options.use_language_detection,
            },
        )?;
        is_binary_file = true;
    }

    // Filter to valid strings if not in verbose mode
    let mut output_results = if options.verbose {
        results.clone()
    } else {
        results.iter().filter(|r| r.is_valid).cloned().collect()
    };

    // Remove duplicates if unique option is specified
    if options.unique {
        let mut seen: std::collections::HashMap<String, StringAnalysisResult> =
            std::collections::HashMap::new();
        let mut unique_results = Vec::new();

        for result in output_results {
            let key = result.original_string.clone();
            if let Some(existing) = seen.get(&key) {
                if result.score > existing.score {
                    seen.insert(key, result.clone());
                }
            } else {
                seen.insert(key, result.clone());
            }
        }

        unique_results.extend(seen.into_values());
        output_results = unique_results;
    }

    // Sort results
    match options.sort.as_str() {
        "score" => output_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap()),
        "alpha" => output_results.sort_by(|a, b| a.original_string.cmp(&b.original_string)),
        "offset" => {
            if is_binary_file {
                output_results.sort_by(|a, b| a.offset.unwrap_or(0).cmp(&b.offset.unwrap_or(0)));
            } else {
                eprintln!("Warning: Offset sorting only available for binary files, sorting by score instead");
                output_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            }
        }
        _ => output_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap()),
    }

    // Output results
    let output_content = format_output(&output_results, options)?;

    if let Some(output_path) = &options.output {
        fs::write(output_path, output_content)?;
        if options.verbose {
            eprintln!("Results written to: {}", output_path);
        }
    } else {
        print!("{}", output_content);
    }

    if options.verbose {
        let valid_count = results.iter().filter(|r| r.is_valid).count();
        let rejected_count = results.len() - valid_count;
        let unique_note = if options.unique {
            format!(" ({} unique shown)", output_results.len())
        } else {
            String::new()
        };

        eprintln!("\nSummary:");
        eprintln!("  Accepted: {} strings", valid_count);
        eprintln!("  Rejected: {} strings", rejected_count);
        eprintln!("  Total: {} strings", results.len());
        eprintln!(
            "  Acceptance rate: {:.1}%{}",
            (valid_count as f64 / results.len() as f64) * 100.0,
            unique_note
        );
    }

    Ok(())
}

fn test_command(options: &CliOptions) -> Result<(), StrangerError> {
    let mut analyzer = StrangerStrings::new();

    analyzer.load_model(&AnalysisOptions {
        model_path: Some(options.model.clone()),
        ..Default::default()
    })?;

    let test_cases = vec![
        (
            "Valid English",
            vec!["hello", "world", "function", "initialize", "process"],
        ),
        (
            "Valid Technical",
            vec!["file_inherit", "total %qu", "Error: %s", "main()", "sizeof"],
        ),
        (
            "Invalid Random",
            vec![".CRT$XIC", "Ta&@", "xZ#@$%", "!@#$%^&*", "}{][++"],
        ),
        ("Edge Cases", vec!["ab", "a", "", "123", "XML"]),
    ];

    println!("=== StrangerStrings Test Results ===\n");

    let (model_type, is_lowercase) = analyzer.get_model_info()?;
    println!("Model: {} (lowercase: {})\n", model_type, is_lowercase);

    for (category, test_strings) in test_cases {
        println!("{}:", category);
        println!("{}", "-".repeat(category.len() + 1));

        for test_string in test_strings {
            let result = analyzer.analyze_string(test_string)?;
            let status = if result.is_valid { "✓" } else { "✗" };

            if options.verbose {
                println!(
                    "  {} \"{}\" → score: {:.3}, threshold: {:.3}",
                    status, test_string, result.score, result.threshold
                );
            } else {
                println!("  {} \"{}\"", status, test_string);
            }
        }
        println!();
    }

    Ok(())
}

fn info_command(options: &CliOptions) -> Result<(), StrangerError> {
    let mut analyzer = StrangerStrings::new();

    if !Path::new(&options.model).exists() {
        return Err(StrangerError::InvalidInput(format!(
            "Model file not found: {}",
            options.model
        )));
    }

    analyzer.load_model(&AnalysisOptions {
        model_path: Some(options.model.clone()),
        ..Default::default()
    })?;

    let (model_type, is_lowercase) = analyzer.get_model_info()?;
    let stats = fs::metadata(&options.model)?;

    println!("=== Model Information ===");
    println!("File: {}", options.model);
    println!("Size: {:.1} KB", stats.len() as f64 / 1024.0);
    println!("Type: {}", model_type);
    println!("Lowercase: {}", is_lowercase);
    println!(
        "Modified: {:?}",
        stats
            .modified()
            .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
    );

    println!("\n=== Threshold Information ===");
    println!("Length-based thresholds:");
    for i in 4..=20 {
        let threshold = get_threshold_for_length(i);
        println!("  Length {:2}: {:.3}", i, threshold);
    }
    println!(
        "  Length 50+: {:.3}",
        NG_THRESHOLDS.get(50).unwrap_or(&MAX_NG_THRESHOLD)
    );
    println!("  Length 100+: {:.3}", MAX_NG_THRESHOLD);

    Ok(())
}

fn format_output(
    results: &[StringAnalysisResult],
    options: &CliOptions,
) -> Result<String, StrangerError> {
    match options.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(results)?;
            Ok(json)
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            let has_offsets = results.iter().any(|r| r.offset.is_some());

            // Write header
            if has_offsets {
                wtr.write_record(&[
                    "string",
                    "score",
                    "threshold",
                    "valid",
                    "normalized",
                    "offset",
                ])?;
            } else {
                wtr.write_record(&["string", "score", "threshold", "valid", "normalized"])?;
            }

            // Write data rows
            for result in results {
                let mut row = vec![
                    result.original_string.clone(),
                    result.score.to_string(),
                    result.threshold.to_string(),
                    result.is_valid.to_string(),
                    result.normalized_string.clone(),
                ];

                if has_offsets {
                    row.push(result.offset.map_or(String::new(), |o| o.to_string()));
                }

                wtr.write_record(&row)?;
            }

            let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
            Ok(data)
        }
        "text" | _ => {
            if options.verbose {
                let has_offsets = results.iter().any(|r| r.offset.is_some());
                let mut output = String::new();

                if has_offsets {
                    output.push_str(&format!(
                        "{:<20} {:<12} {:<12} {:<10} {}\n",
                        "String", "Score", "Threshold", "Offset", "Valid"
                    ));
                    output.push_str(&"-".repeat(70));
                } else {
                    output.push_str(&format!(
                        "{:<20} {:<12} {:<12} {}\n",
                        "String", "Score", "Threshold", "Valid"
                    ));
                    output.push_str(&"-".repeat(60));
                }
                output.push('\n');

                for result in results {
                    let status = if result.is_valid { "✓" } else { "✗" };
                    let string_display = format!("\"{}\"", result.original_string);

                    if has_offsets {
                        let offset_display = result
                            .offset
                            .map_or(String::new(), |o| format!("0x{:X}", o));
                        output.push_str(&format!(
                            "{:<20} {:<12.3} {:<12.3} {:<10} {}\n",
                            string_display, result.score, result.threshold, offset_display, status
                        ));
                    } else {
                        output.push_str(&format!(
                            "{:<20} {:<12.3} {:<12.3} {}\n",
                            string_display, result.score, result.threshold, status
                        ));
                    }
                }

                Ok(output)
            } else {
                let output = results
                    .iter()
                    .map(|r| r.original_string.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(if output.is_empty() {
                    output
                } else {
                    output + "\n"
                })
            }
        }
    }
}
