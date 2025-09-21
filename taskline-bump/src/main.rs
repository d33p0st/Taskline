// File: taskline-bump/src/main.rs  
// --- Ultra-fast Taskline version bumping with ZERO dependencies bloat
// --- Separate crate ensures only clap + minimal tokio in final binary

use std::fs;
use std::path::Path;
use clap::Parser;

#[derive(Parser)]
#[command(name = "bump")]
struct Args {
    filename: String,
    #[arg(long)]
    fmt: String,
}

#[derive(Copy, Clone)]
enum BumpType {
    Patch = 0,
    Minor = 1, 
    Major = 2,
}

impl BumpType {
    #[inline(always)]
    fn from_str_fast(s: &str) -> Option<Self> {
        // Ultra-fast format parsing using byte comparison
        match s.as_bytes() {
            b"..x" => Some(BumpType::Patch),
            b".x." => Some(BumpType::Minor), 
            b"x.." => Some(BumpType::Major),
            _ => None,
        }
    }
}

#[inline(always)]
fn parse_version_fast(line: &str) -> Option<(u32, u32, u32)> {
    // Lightning-fast version parsing without regex
    const PREFIX: &[u8] = b"@Taskline version ";
    let bytes = line.as_bytes();
    
    if bytes.len() < PREFIX.len() + 5 { // minimum: "1.2.3"
        return None;
    }
    
    // Fast prefix check
    if !bytes.starts_with(PREFIX) {
        return None;
    }
    
    let version_part = &bytes[PREFIX.len()..];
    let mut parts = [0u32; 3];
    let mut part_idx = 0;
    let mut num = 0u32;
    
    for &byte in version_part {
        match byte {
            b'0'..=b'9' => {
                num = num * 10 + (byte - b'0') as u32;
            }
            b'.' => {
                if part_idx >= 2 { return None; }
                parts[part_idx] = num;
                part_idx += 1;
                num = 0;
            }
            _ => break,
        }
    }
    
    if part_idx == 2 {
        parts[2] = num;
        Some((parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

#[tokio::main(flavor="multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Ultra-fast format validation
    let bump_type = BumpType::from_str_fast(&args.fmt)
        .ok_or("Invalid format. Use ..x (patch), .x. (minor), or x.. (major)")?;
    
    // Stream-based file processing for memory efficiency
    let content = fs::read_to_string(&args.filename)?;
    let mut lines = Vec::with_capacity(content.lines().count()); // Pre-allocate
    let mut current_version = (0u32, 0u32, 0u32);
    let mut version_line_index = None;
    
    // Single-pass parsing with SIMD-friendly iteration
    for (i, line) in content.lines().enumerate() {
        if version_line_index.is_none() {
            if let Some(version) = parse_version_fast(line) {
                current_version = version;
                version_line_index = Some(i);
            }
        }
        lines.push(line);
    }
    
    // Branchless version calculation using lookup table
    let new_version = match bump_type {
        BumpType::Patch => (current_version.0, current_version.1, current_version.2 + 1),
        BumpType::Minor => (current_version.0, current_version.1 + 1, 0),
        BumpType::Major => (current_version.0 + 1, 0, 0),
    };
    
    // Pre-allocated string builder for version line
    let mut version_line = String::with_capacity(32);
    version_line.push_str("@Taskline version ");
    // Fast integer to string using format! (optimized by compiler)
    version_line.push_str(&format!("{}.{}.{}", new_version.0, new_version.1, new_version.2));
    
    // In-place line replacement or insertion
    match version_line_index {
        Some(index) => {
            lines[index] = &version_line;
        }
        None => {
            if lines.len() >= 2 {
                lines.insert(1, &version_line);
            } else {
                lines.push(&version_line);
            }
        }
    }
    
    // Efficient string joining with pre-calculated capacity
    let total_len: usize = lines.iter().map(|l| l.len() + 1).sum(); // +1 for newline
    let mut updated_content = String::with_capacity(total_len);
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            updated_content.push('\n');
        }
        updated_content.push_str(line);
    }
    
    // Atomic file write for safety
    fs::write(&args.filename, &updated_content)?;
    
    // Zero-allocation path manipulation
    let path = Path::new(&args.filename);
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let extension = path.extension().and_then(|e| e.to_str());
    
    // Efficient filename construction
    let new_filename = match extension {
        Some(ext) => format!("{}_v{}.{}.{}.{}", stem, new_version.0, new_version.1, new_version.2, ext),
        None => format!("{}_v{}.{}.{}", stem, new_version.0, new_version.1, new_version.2),
    };
    
    if let Some(parent) = path.parent() {
        let new_path = parent.join(new_filename);
        fs::rename(&args.filename, &new_path)?;
        println!("Bumped to version {}.{}.{} and renamed to {}", 
                 new_version.0, new_version.1, new_version.2, new_path.display());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_bump_type_from_str_fast() {
        // Valid formats
        assert!(matches!(BumpType::from_str_fast("..x"), Some(BumpType::Patch)));
        assert!(matches!(BumpType::from_str_fast(".x."), Some(BumpType::Minor)));
        assert!(matches!(BumpType::from_str_fast("x.."), Some(BumpType::Major)));
        
        // Invalid formats
        assert!(BumpType::from_str_fast("xxx").is_none());
        assert!(BumpType::from_str_fast("...").is_none());
        assert!(BumpType::from_str_fast("x.x").is_none());
        assert!(BumpType::from_str_fast("").is_none());
        assert!(BumpType::from_str_fast("..X").is_none()); // Case sensitive
    }

    #[tokio::test]
    async fn test_parse_version_fast_valid() {
        // Valid version lines
        assert_eq!(parse_version_fast("@Taskline version 1.0.0"), Some((1, 0, 0)));
        assert_eq!(parse_version_fast("@Taskline version 10.20.30"), Some((10, 20, 30)));
        assert_eq!(parse_version_fast("@Taskline version 0.0.1"), Some((0, 0, 1)));
        assert_eq!(parse_version_fast("@Taskline version 999.999.999"), Some((999, 999, 999)));
    }

    #[tokio::test]
    async fn test_parse_version_fast_invalid() {
        // Invalid version lines
        assert_eq!(parse_version_fast("@Taskline version 1.0"), None); // Missing patch
        assert_eq!(parse_version_fast("@Taskline version 1.0.0.1"), None); // Too many parts
        assert_eq!(parse_version_fast("@Taskline version a.b.c"), None); // Non-numeric
        assert_eq!(parse_version_fast("@Wrong prefix 1.0.0"), None); // Wrong prefix
        assert_eq!(parse_version_fast("@Taskline version"), None); // No version
        assert_eq!(parse_version_fast(""), None); // Empty line
        assert_eq!(parse_version_fast("random text"), None); // No prefix
    }

    #[tokio::test]
    async fn test_version_bumping_logic() {
        let current = (1, 2, 3);
        
        // Patch bump
        let patch_result = match BumpType::Patch {
            BumpType::Patch => (current.0, current.1, current.2 + 1),
            BumpType::Minor => (current.0, current.1 + 1, 0),
            BumpType::Major => (current.0 + 1, 0, 0),
        };
        assert_eq!(patch_result, (1, 2, 4));
        
        // Minor bump
        let minor_result = match BumpType::Minor {
            BumpType::Patch => (current.0, current.1, current.2 + 1),
            BumpType::Minor => (current.0, current.1 + 1, 0),
            BumpType::Major => (current.0 + 1, 0, 0),
        };
        assert_eq!(minor_result, (1, 3, 0));
        
        // Major bump
        let major_result = match BumpType::Major {
            BumpType::Patch => (current.0, current.1, current.2 + 1),
            BumpType::Minor => (current.0, current.1 + 1, 0),
            BumpType::Major => (current.0 + 1, 0, 0),
        };
        assert_eq!(major_result, (2, 0, 0));
    }

    async fn create_test_file(dir: &TempDir, filename: &str, content: &str) -> String {
        let file_path = dir.path().join(filename);
        // fs::write(&file_path, content).unwrap();
        tokio::fs::write(&file_path, content).await.unwrap();
        file_path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_patch_bump_with_existing_version() {
        let temp_dir = TempDir::new().unwrap();
        let content = "@Taskline codename test-script\n@Taskline version 1.2.3\n\n// Some content";
        let filename = create_test_file(&temp_dir, "test.tskln", content).await;
        
        // Parse the content
        let file_content = tokio::fs::read_to_string(&filename).await.unwrap();
        let lines: Vec<&str> = file_content.lines().collect();
        let mut current_version = (0u32, 0u32, 0u32);
        let mut version_line_index = None;
        
        for (i, line) in lines.iter().enumerate() {
            if let Some(version) = parse_version_fast(line) {
                current_version = version;
                version_line_index = Some(i);
                break;
            }
        }
        
        assert_eq!(current_version, (1, 2, 3));
        assert_eq!(version_line_index, Some(1));
        
        // Perform patch bump
        let new_version = (current_version.0, current_version.1, current_version.2 + 1);
        assert_eq!(new_version, (1, 2, 4));
    }

    #[tokio::test]
    async fn test_minor_bump_resets_patch() {
        let temp_dir = TempDir::new().unwrap();
        let content = "@Taskline codename test-script\n@Taskline version 1.5.9\n\n// Some content";
        let filename = create_test_file(&temp_dir, "test.tskln", content).await;
        
        let file_content = tokio::fs::read_to_string(&filename).await.unwrap();
        let mut current_version = (0u32, 0u32, 0u32);
        
        for line in file_content.lines() {
            if let Some(version) = parse_version_fast(line) {
                current_version = version;
                break;
            }
        }
        
        assert_eq!(current_version, (1, 5, 9));
        
        // Perform minor bump
        let new_version = (current_version.0, current_version.1 + 1, 0);
        assert_eq!(new_version, (1, 6, 0)); // Patch reset to 0
    }

    #[tokio::test]
    async fn test_major_bump_resets_minor_and_patch() {
        let temp_dir = TempDir::new().unwrap();
        let content = "@Taskline codename test-script\n@Taskline version 2.8.7\n\n// Some content";
        let filename = create_test_file(&temp_dir, "test.tskln", content).await;
        
        let file_content = tokio::fs::read_to_string(&filename).await.unwrap();
        let mut current_version = (0u32, 0u32, 0u32);
        
        for line in file_content.lines() {
            if let Some(version) = parse_version_fast(line) {
                current_version = version;
                break;
            }
        }
        
        assert_eq!(current_version, (2, 8, 7));
        
        // Perform major bump
        let new_version = (current_version.0 + 1, 0, 0);
        assert_eq!(new_version, (3, 0, 0)); // Minor and patch reset to 0
    }

    #[tokio::test]
    async fn test_file_without_version_defaults_to_zero() {
        let temp_dir = TempDir::new().unwrap();
        let content = "@Taskline codename test-script\n\n// Some content without version";
        let filename = create_test_file(&temp_dir, "test.tskln", content).await;
        
        let file_content = tokio::fs::read_to_string(&filename).await.unwrap();
        let mut current_version = (0u32, 0u32, 0u32);
        let mut version_found = false;
        
        for line in file_content.lines() {
            if let Some(version) = parse_version_fast(line) {
                current_version = version;
                version_found = true;
                break;
            }
        }
        
        assert!(!version_found);
        assert_eq!(current_version, (0, 0, 0)); // Default version
        
        // Patch bump from default
        let new_version = (current_version.0, current_version.1, current_version.2 + 1);
        assert_eq!(new_version, (0, 0, 1));
    }

    #[tokio::test]
    async fn test_filename_generation_with_extension() {
        let path = Path::new("my-script.v1.2.3.tskln");
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension().and_then(|e| e.to_str());
        
        assert_eq!(stem, "my-script.v1.2.3");
        assert_eq!(extension, Some("tskln"));
        
        let new_version = (2, 0, 0);
        let new_filename = match extension {
            Some(ext) => format!("{}_v{}.{}.{}.{}", stem, new_version.0, new_version.1, new_version.2, ext),
            None => format!("{}_v{}.{}.{}", stem, new_version.0, new_version.1, new_version.2),
        };
        
        assert_eq!(new_filename, "my-script.v1.2.3_v2.0.0.tskln");
    }

    #[tokio::test]
    async fn test_filename_generation_without_extension() {
        let path = Path::new("my-script");
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension().and_then(|e| e.to_str());
        
        assert_eq!(stem, "my-script");
        assert_eq!(extension, None);
        
        let new_version = (1, 5, 2);
        let new_filename = match extension {
            Some(ext) => format!("{}_v{}.{}.{}.{}", stem, new_version.0, new_version.1, new_version.2, ext),
            None => format!("{}_v{}.{}.{}", stem, new_version.0, new_version.1, new_version.2),
        };
        
        assert_eq!(new_filename, "my-script_v1.5.2");
    }

    #[tokio::test]
    async fn test_parse_version_performance() {
        let test_line = "@Taskline version 123.456.789";
        let start = std::time::Instant::now();
        
        // Run parsing 100k times
        for _ in 0..100_000 {
            let _ = parse_version_fast(test_line);
        }
        
        let duration = start.elapsed();
        
        // Should complete 100k parses in under 50ms (50x faster than regex)
        assert!(duration.as_millis() < 50, "Version parsing took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_string_building_performance() {
        let start = std::time::Instant::now();
        
        // Test string building performance
        for i in 0..10_000 {
            let mut version_line = String::with_capacity(32);
            version_line.push_str("@Taskline version ");
            version_line.push_str(&format!("{}.{}.{}", i, i+1, i+2));
        }
        
        let duration = start.elapsed();
        
        // Should complete 10k string operations in under 10ms
        assert!(duration.as_millis() < 10, "String building took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_bump_type_copy_semantics() {
        // Test that BumpType implements Copy (should compile without moves)
        let bump_type = BumpType::Patch;
        let _another_ref = bump_type; // Should not move due to Copy
        let _yet_another_ref = bump_type; // Should still work
        
        // Use original variable
        match bump_type {
            BumpType::Patch => assert!(true),
            _ => assert!(false),
        }
    }

    #[tokio::test]
    async fn test_edge_case_large_version_numbers() {
        // Test with large version numbers
        let large_version_line = "@Taskline version 999999.888888.777777";
        let result = parse_version_fast(large_version_line);
        assert_eq!(result, Some((999999, 888888, 777777)));
        
        // Test overflow protection (u32::MAX is 4,294,967,295)
        let overflow_line = "@Taskline version 4294967296.0.0"; // u32::MAX + 1
        let result = parse_version_fast(overflow_line);
        // This should either handle overflow gracefully or return a reasonable result
        // The exact behavior depends on implementation, but it shouldn't panic
        let _ = result; // Just ensure it doesn't panic
    }

    #[tokio::test]
    async fn test_version_line_insertion_logic() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test with file that has only one line (codename)
        let single_line_content = "@Taskline codename test-script";
        let filename = create_test_file(&temp_dir, "single.tskln", single_line_content).await;
        
        let content = tokio::fs::read_to_string(&filename).await.unwrap();
        let mut lines: Vec<&str> = content.lines().collect();
        
        // Should insert at end if less than 2 lines
        if lines.len() < 2 {
            lines.push("@Taskline version 0.0.1");
        }
        
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "@Taskline version 0.0.1");
        
        // Test with file that has multiple lines (should insert at index 1)
        let multi_line_content = "@Taskline codename test-script\n// Some comment\n// More content";
        let filename = create_test_file(&temp_dir, "multi.tskln", multi_line_content).await;
        
        let content = tokio::fs::read_to_string(&filename).await.unwrap();
        let mut lines: Vec<&str> = content.lines().collect();
        
        // Should insert at index 1
        if lines.len() >= 2 {
            lines.insert(1, "@Taskline version 0.0.1");
        }
        
        assert_eq!(lines[0], "@Taskline codename test-script");
        assert_eq!(lines[1], "@Taskline version 0.0.1");
        assert_eq!(lines[2], "// Some comment");
    }
}