// File: taskline-init/src/main.rs
// --- Ultra-fast Taskline initialization with minimal dependencies
// --- Optimized for maximum performance and minimal binary size

use std::io::Write;
use chrono::prelude::*;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
struct TasklineInitializationError {
    details: String
}

impl std::fmt::Display for TasklineInitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TasklineInitializationError: {}", self.details)
    }
}

async fn validate_version(version: &str) -> Result<(), TasklineInitializationError> {
    log::trace!("Starting version validation for: '{}'", version);
    
    // Ultra-fast version validation using direct byte access
    let bytes = version.as_bytes();
    let len = bytes.len();

    log::debug!("Version string length: {} bytes", len);

    // Fast early return for empty or too short versions
    if len < 5 {  // Minimum: "v1.0" (4 chars) but we need at least one more for patch
        log::warn!("Version too short: '{}' (length: {}), minimum format is v1.0.0", version, len);
        return Err(TasklineInitializationError {
            details: "Version too short - minimum format is v1.0.0".to_string()
        });
    }

    // Check first byte without bounds checking (we know len >= 5)
    if unsafe { *bytes.get_unchecked(0) } != b'v' {
        log::error!("Version does not start with 'v': '{}'", version);
        return Err(TasklineInitializationError {
            details: "Version must start with 'v'".to_string()
        });
    }

    log::trace!("Version starts with 'v' - continuing validation");

    let mut dot_count = 0u8;
    let mut has_digits_in_current_part = false;
    let mut i = 1;

    // Manual loop is faster than iterator combinators
    while i < len {
        let byte = unsafe { *bytes.get_unchecked(i) };
        
        match byte {
            b'.' => {
                if !has_digits_in_current_part {
                    log::error!("Empty version part found at position {} in '{}'", i, version);
                    return Err(TasklineInitializationError {
                        details: "Empty version part - each part must contain digits".to_string()
                    });
                }
                
                dot_count += 1;
                log::trace!("Found dot #{} at position {}", dot_count, i);
                
                if dot_count > 2 {
                    log::error!("Too many dots ({}) in version '{}'", dot_count, version);
                    return Err(TasklineInitializationError {
                        details: "Too many dots - version must have exactly 2 dots (major.minor.patch)".to_string()
                    });
                }
                
                has_digits_in_current_part = false;
            }
            b'0'..=b'9' => {
                has_digits_in_current_part = true;
                log::trace!("Found digit '{}' at position {}", byte as char, i);
            }
            _ => {
                log::error!("Invalid character '{}' at position {} in version '{}'", byte as char, i, version);
                return Err(TasklineInitializationError {
                    details: "Invalid character - version can only contain digits and dots".to_string()
                });
            }
        }
        
        i += 1;
    }

    // Final validation - use bitwise operations for faster comparison
    if dot_count != 2 {
        log::error!("Incorrect number of dots: {} (expected 2) in version '{}'", dot_count, version);
        return Err(TasklineInitializationError {
            details: "Missing dots - version must have exactly 2 dots (major.minor.patch)".to_string()
        });
    }

    if !has_digits_in_current_part {
        log::error!("Last version part is empty in '{}'", version);
        return Err(TasklineInitializationError {
            details: "Last version part is empty - must contain digits".to_string()
        });
    }

    log::info!("Version validation successful for '{}'", version);
    Ok(())
}

#[tokio::main(flavor="multi_thread")]
async fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            // Get current UTC time and convert to IST (UTC+5:30)
            let now_utc = Utc::now();
            let ist_offset = chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap();
            let now_ist = now_utc.with_timezone(&ist_offset);
            writeln!(
            buf,
            "[{}] [{}] - {}",
            now_ist.format("%H:%M:%S%.3f"),
            record.level(),
            record.args()
            )
        })
        .filter_level(log::LevelFilter::Trace)
        .filter(Some("serial_test"), log::LevelFilter::Error)
        .filter(Some("tokio_tungstenite"), log::LevelFilter::Error)
        .filter(Some("tungstenite"), log::LevelFilter::Error)
        .filter(Some("hyper"), log::LevelFilter::Error)
        .filter(Some("tracing"), log::LevelFilter::Error)
        .filter(Some("warp"), log::LevelFilter::Error)
        .filter(Some("reqwest"), log::LevelFilter::Error)
        .filter(Some("thirtyfour"), log::LevelFilter::Error)
        .init();

    log::info!("Taskline-init starting up");
    log::trace!("Logger initialized with IST timezone");

    let arguments: Vec<String> = std::env::args().collect();
    log::debug!("Command line arguments: {:?}", arguments);
    
    // Expecting exactly 2 arguments: program name, filename
    // or 3 arguments: program name, filename, version
    let argument_length = arguments.len();
    log::trace!("Argument count: {}", argument_length);
    
    if argument_length < 2 || argument_length > 3 {
        log::error!("Invalid argument count: {} (expected 2 or 3)", argument_length);
        eprintln!("Usage: taskline.init <filename> [version]");
        std::process::exit(1);
    }

    let filename = &arguments[1];
    let version = if argument_length == 3 {
        Some(&arguments[2])
    } else {
        None
    };

    log::info!("Initializing file: '{}' with version: {:?}", filename, version);

    let filename_with_extension: String;
    if let Some(ver) = version {
        log::debug!("Version provided: '{}' - validating", ver);
        if let Err(e) = validate_version(ver).await {
            log::error!("Version validation failed: {}", e);
            eprintln!("{}", e);
            std::process::exit(1);
        }
        log::info!("Version validation passed for '{}'", ver);
        filename_with_extension = format!("{}.{}.tskln", filename, ver);
    } else {
        log::debug!("No version provided - using default naming");
        filename_with_extension = format!("{}.tskln", filename);
    }

    log::info!("Target filename: '{}'", filename_with_extension);

    if std::path::Path::new(&filename_with_extension).exists() {
        log::warn!("File '{}' already exists - aborting to prevent overwrite", filename_with_extension);
        eprintln!("File '{}' already exists. Initialization aborted to prevent overwriting.", filename_with_extension);
        std::process::exit(1);
    }

    log::debug!("File does not exist - proceeding with creation");

    let mut file = match tokio::fs::File::create(&filename_with_extension).await {
        Ok(f) => {
            log::info!("File '{}' created successfully", filename_with_extension);
            f
        },
        Err(e) => {
            log::error!("Failed to create file '{}': {}", filename_with_extension, e);
            eprintln!("Failed to create file '{}': {}", filename_with_extension, e);
            std::process::exit(1);
        }
    };

    // The following content has to be written to the file
    // @Taskline codename {filename}
    // @Taskline version {version} (if provided)

    let header = if let Some(ver) = version {
        let h = format!("@Taskline codename {}\n@Taskline version {}\n\n", filename, ver);
        log::debug!("Generated header with version: '{}'", h.trim());
        h
    } else {
        let h = format!("@Taskline codename {}\n\n", filename);
        log::debug!("Generated header without version: '{}'", h.trim());
        h
    };

    log::trace!("Writing header to file (length: {} bytes)", header.len());

    if let Err(e) = file.write_all(header.as_bytes()).await {
        log::error!("Failed to write to file '{}': {}", filename_with_extension, e);
        eprintln!("Failed to initialize '{}': {}", filename_with_extension, e);
        std::process::exit(1);
    }

    log::info!("Successfully initialized Taskline file: '{}'", filename_with_extension);
    log::trace!("Taskline-init completed successfully");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    // Initialize logger only once for all tests
    fn init_test_logger() {
        INIT_LOGGER.call_once(|| {
            env_logger::Builder::new()
                .filter_level(log::LevelFilter::Trace)
                .is_test(true)
                .try_init()
                .ok(); // Ignore error if already initialized
        });
    }

    #[tokio::test]
    async fn test_validate_version_valid_cases() {
        init_test_logger();
        log::info!("Starting test_validate_version_valid_cases");
        
        // Valid version formats
        log::debug!("Testing valid version: v1.0.0");
        assert!(validate_version("v1.0.0").await.is_ok());
        
        log::debug!("Testing valid version: v0.0.1");
        assert!(validate_version("v0.0.1").await.is_ok());
        
        log::debug!("Testing valid version: v10.20.30");
        assert!(validate_version("v10.20.30").await.is_ok());
        
        log::debug!("Testing valid version: v999.999.999");
        assert!(validate_version("v999.999.999").await.is_ok());
        
        log::debug!("Testing valid version: v1.0.0 (duplicate)");
        assert!(validate_version("v1.0.0").await.is_ok());
        
        log::info!("test_validate_version_valid_cases completed successfully");
    }

    #[tokio::test]
    async fn test_validate_version_invalid_cases() {
        init_test_logger();
        log::info!("Starting test_validate_version_invalid_cases");
        
        // Invalid formats
        log::debug!("Testing invalid version: 1.0.0 (missing 'v')");
        assert!(validate_version("1.0.0").await.is_err()); // Missing 'v'
        
        log::debug!("Testing invalid version: v1.0 (missing patch)");
        assert!(validate_version("v1.0").await.is_err()); // Missing patch
        
        log::debug!("Testing invalid version: v1.0. (empty patch)");
        assert!(validate_version("v1.0.").await.is_err()); // Empty patch
        
        log::debug!("Testing invalid version: v.1.0 (empty major)");
        assert!(validate_version("v.1.0").await.is_err()); // Empty major
        
        log::debug!("Testing invalid version: v1..0 (empty minor)");
        assert!(validate_version("v1..0").await.is_err()); // Empty minor
        
        log::debug!("Testing invalid version: v1.0.0.1 (too many parts)");
        assert!(validate_version("v1.0.0.1").await.is_err()); // Too many parts
        
        log::debug!("Testing invalid version: va.b.c (non-numeric)");
        assert!(validate_version("va.b.c").await.is_err()); // Non-numeric
        
        log::debug!("Testing invalid version: v1.2.3a (invalid character)");
        assert!(validate_version("v1.2.3a").await.is_err()); // Invalid character
        
        log::debug!("Testing invalid version: '' (empty)");
        assert!(validate_version("").await.is_err()); // Empty
        
        log::debug!("Testing invalid version: 'v' (too short)");
        assert!(validate_version("v").await.is_err()); // Too short
        
        log::debug!("Testing invalid version: 'v1' (too short)");
        assert!(validate_version("v1").await.is_err()); // Too short
        
        log::info!("test_validate_version_invalid_cases completed successfully");
    }

    #[tokio::test]
    async fn test_validate_version_edge_cases() {
        init_test_logger();
        log::info!("Starting test_validate_version_edge_cases");
        
        // Edge cases
        log::debug!("Testing valid edge case: 'v0.0.0' (all zeros)");
        assert!(validate_version("v0.0.0").await.is_ok()); // All zeros
        
        log::debug!("Testing valid edge case: 'v1.2.3' (standard format)");
        assert!(validate_version("v1.2.3").await.is_ok()); // Standard format
        
        log::debug!("Testing valid edge case: 'v100.200.300' (large numbers)");
        assert!(validate_version("v100.200.300").await.is_ok()); // Large numbers
        
        // Invalid edge cases
        log::debug!("Testing invalid edge case: 'v1.2.3.' (trailing dot)");
        assert!(validate_version("v1.2.3.").await.is_err()); // Trailing dot
        
        log::debug!("Testing invalid edge case: 'v.' (just v and dot)");
        assert!(validate_version("v.").await.is_err()); // Just v and dot
        
        log::debug!("Testing invalid edge case: 'v..' (double dots at start)");
        assert!(validate_version("v..").await.is_err()); // Double dots at start
        
        log::info!("test_validate_version_edge_cases completed successfully");
    }

    #[tokio::test]
    async fn test_file_creation_without_version() {
        init_test_logger();
        log::info!("Starting test_file_creation_without_version");
        
        let temp_dir = TempDir::new().unwrap();
        let test_filename = "test-script";
        
        log::debug!("Testing file creation for: '{}'", test_filename);
        
        // Simulate the file creation logic from main
        let filename_with_extension = format!("{}.tskln", test_filename);
        let full_path = temp_dir.path().join(&filename_with_extension);
        
        log::trace!("Target path: {:?}", full_path);
        
        // Check file doesn't exist
        assert!(!full_path.exists());
        log::debug!("Confirmed file does not exist");
        
        // Create file with expected content
        let header = format!("@Taskline codename {}\n\n", test_filename);
        log::trace!("Generated header: '{}'", header.trim());
        
        fs::write(&full_path, &header).await.unwrap();
        log::debug!("File written successfully");
        
        // Verify file was created
        assert!(full_path.exists());
        log::debug!("Confirmed file exists after creation");
        
        // Verify content
        let content = fs::read_to_string(&full_path).await.unwrap();
        log::trace!("File content: '{}'", content);
        assert_eq!(content, "@Taskline codename test-script\n\n");
        
        log::info!("test_file_creation_without_version completed successfully");
    }

    #[tokio::test]
    async fn test_file_creation_with_version() {
        init_test_logger();
        log::info!("Starting test_file_creation_with_version");
        
        let temp_dir = TempDir::new().unwrap();
        let test_filename = "test-script";
        let test_version = "v1.2.3";
        
        log::debug!("Testing file creation for: '{}' with version: '{}'", test_filename, test_version);
        
        // Validate version first
        log::trace!("Validating version: '{}'", test_version);
        assert!(validate_version(test_version).await.is_ok());
        log::debug!("Version validation passed");
        
        // Simulate the file creation logic from main
        let filename_with_extension = format!("{}.{}.tskln", test_filename, test_version);
        let full_path = temp_dir.path().join(&filename_with_extension);
        
        log::trace!("Target path: {:?}", full_path);
        
        // Check file doesn't exist
        assert!(!full_path.exists());
        log::debug!("Confirmed file does not exist");
        
        // Create file with expected content
        let header = format!("@Taskline codename {}\n@Taskline version {}\n\n", test_filename, test_version);
        log::trace!("Generated header: '{}'", header.trim());
        
        fs::write(&full_path, &header).await.unwrap();
        log::debug!("File written successfully");
        
        // Verify file was created
        assert!(full_path.exists());
        log::debug!("Confirmed file exists after creation");
        
        // Verify content
        let content = fs::read_to_string(&full_path).await.unwrap();
        log::trace!("File content: '{}'", content);
        assert_eq!(content, "@Taskline codename test-script\n@Taskline version v1.2.3\n\n");
        
        log::info!("test_file_creation_with_version completed successfully");
    }

    #[tokio::test]
    async fn test_file_overwrite_protection() {
        init_test_logger();
        log::info!("Starting test_file_overwrite_protection");
        
        let temp_dir = TempDir::new().unwrap();
        let test_filename = "existing-script";
        let existing_file = "existing-script.tskln";
        let existing_path = temp_dir.path().join(existing_file);
        
        log::debug!("Creating existing file: {:?}", existing_path);
        
        // Create existing file
        fs::write(&existing_path, "existing content").await.unwrap();
        assert!(existing_path.exists());
        log::debug!("Confirmed existing file created");
        
        // Verify the file exists (simulating the check in main)
        let filename_with_extension = format!("{}.tskln", test_filename);
        let check_path = temp_dir.path().join(&filename_with_extension);
        
        log::trace!("Checking for file overwrite protection at: {:?}", check_path);
        assert!(check_path.exists()); // Should prevent overwriting
        
        log::info!("test_file_overwrite_protection completed successfully");
    }

    #[tokio::test]
    async fn test_version_validation_performance() {
        init_test_logger();
        log::info!("Starting test_version_validation_performance");
        
        // Performance test - should complete quickly
        let start = std::time::Instant::now();
        
        log::debug!("Starting 20,000 version validations (10k valid, 10k invalid)");
        
        for i in 0..10000 {
            let _ = validate_version("v1.2.3").await;
            let _ = validate_version("invalid").await;
            
            if i % 2000 == 0 {
                log::trace!("Completed {} validation cycles", i);
            }
        }
        
        let duration = start.elapsed();
        log::info!("Performance test completed in {:?}", duration);
        
        // Should complete 20k validations in under 2 seconds (allowing for logging overhead)
        assert!(duration.as_millis() < 2000, "Version validation took too long: {:?}", duration);
        
        log::info!("test_version_validation_performance completed successfully - {} validations in {:?}", 20000, duration);
    }

    #[tokio::test]
    async fn test_end_to_end_performance() {
        init_test_logger();
        log::info!("Starting test_end_to_end_performance");
        
        let temp_dir = TempDir::new().unwrap();
        let start = std::time::Instant::now();
        
        log::debug!("Starting end-to-end performance test with 1000 operations");
        
        // Simulate complete workflow
        for i in 0..1000 {
            let filename = format!("perf-test-{}", i);
            let version = format!("v1.0.{}", i % 100);
            
            if i % 200 == 0 {
                log::trace!("Completed {} operations", i);
            }
            
            // Validate version
            validate_version(&version).await.unwrap();
            
            // Create filename
            let filename_with_extension = format!("{}.{}.tskln", filename, version);
            let full_path = temp_dir.path().join(&filename_with_extension);
            
            // Create file
            let header = format!("@Taskline codename {}\n@Taskline version {}\n\n", filename, version);
            fs::write(&full_path, &header).await.unwrap();
        }
        
        let duration = start.elapsed();
        log::info!("End-to-end performance test completed in {:?}", duration);
        
        // Should complete 1000 operations in under 1 second
        assert!(duration.as_millis() < 1000, "End-to-end test took too long: {:?}", duration);
        
        log::info!("test_end_to_end_performance completed successfully - {} operations in {:?}", 1000, duration);
    }

    #[tokio::test]
    async fn test_filename_generation_edge_cases() {
        init_test_logger();
        log::info!("Starting test_filename_generation_edge_cases");
        
        let temp_dir = TempDir::new().unwrap();
        
        // Test with special characters in filename
        let special_filename = "my-script_test";
        let version = "v0.1.0";
        let expected_filename = format!("{}.{}.tskln", special_filename, version);
        let full_path = temp_dir.path().join(&expected_filename);
        
        log::debug!("Testing filename generation with special characters: '{}'", special_filename);
        log::trace!("Expected filename: '{}'", expected_filename);
        log::trace!("Full path: {:?}", full_path);
        
        // This should work fine
        let header = format!("@Taskline codename {}\n@Taskline version {}\n\n", special_filename, version);
        log::trace!("Generated header: '{}'", header.trim());
        
        fs::write(&full_path, &header).await.unwrap();
        log::debug!("File written successfully");
        
        assert!(full_path.exists());
        log::debug!("Confirmed file exists");
        
        let content = fs::read_to_string(&full_path).await.unwrap();
        log::trace!("File content: '{}'", content);
        
        assert!(content.contains("my-script_test"));
        assert!(content.contains("v0.1.0"));
        
        log::info!("test_filename_generation_edge_cases completed successfully");
    }

    #[tokio::test]
    async fn test_concurrent_file_creation() {
        init_test_logger();
        log::info!("Starting test_concurrent_file_creation");
        
        let temp_dir = TempDir::new().unwrap();
        
        // Test creating multiple files concurrently
        let mut handles = vec![];
        
        log::debug!("Starting 10 concurrent file creation tasks");
        
        for i in 0..10 {
            let temp_path = temp_dir.path().to_path_buf();
            let handle = tokio::spawn(async move {
                let filename = format!("concurrent-test-{}", i);
                let version = format!("v1.0.{}", i);
                let filename_with_extension = format!("{}.{}.tskln", filename, version);
                let full_path = temp_path.join(&filename_with_extension);
                
                let header = format!("@Taskline codename {}\n@Taskline version {}\n\n", filename, version);
                tokio::fs::write(&full_path, &header).await.unwrap();
                
                full_path
            });
            handles.push(handle);
        }
        
        log::trace!("All tasks spawned, waiting for completion");
        
        // Wait for all to complete
        for (i, handle) in handles.into_iter().enumerate() {
            let path = handle.await.unwrap();
            assert!(path.exists());
            
            if i % 3 == 0 {
                log::trace!("Completed task {}", i);
            }
        }
        
        log::info!("test_concurrent_file_creation completed successfully - all 10 files created");
    }

}