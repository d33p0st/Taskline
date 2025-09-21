// File: src/lib.rs
// --- Core Taskline library for shared functionality across tools
// --- Ultra-fast common operations and data structures

/// Core error type for Taskline operations
#[derive(Debug, Clone)]
pub enum TasklineError {
    VersionError(String),
    FileError(String),
    ParseError(String),
}

impl std::fmt::Display for TasklineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TasklineError::VersionError(msg) => write!(f, "Version Error: {}", msg),
            TasklineError::FileError(msg) => write!(f, "File Error: {}", msg),
            TasklineError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
        }
    }
}

impl std::error::Error for TasklineError {}

/// Ultra-fast version representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Create a new version
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    /// Parse version from string (ultra-fast implementation)
    pub fn parse(version_str: &str) -> Result<Self, TasklineError> {
        if !version_str.starts_with('v') {
            return Err(TasklineError::VersionError("Version must start with 'v'".to_string()));
        }
        
        let parts: Vec<&str> = version_str[1..].split('.').collect();
        if parts.len() != 3 {
            return Err(TasklineError::VersionError("Version must have format v1.2.3".to_string()));
        }
        
        let major = parts[0].parse().map_err(|_| TasklineError::VersionError("Invalid major version".to_string()))?;
        let minor = parts[1].parse().map_err(|_| TasklineError::VersionError("Invalid minor version".to_string()))?;
        let patch = parts[2].parse().map_err(|_| TasklineError::VersionError("Invalid patch version".to_string()))?;
        
        Ok(Version::new(major, minor, patch))
    }
    
    /// Bump patch version
    pub fn bump_patch(self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
    
    /// Bump minor version (resets patch)
    pub fn bump_minor(self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }
    
    /// Bump major version (resets minor and patch)
    pub fn bump_major(self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Taskline file metadata
#[derive(Debug, Clone)]
pub struct TasklineMetadata {
    pub codename: String,
    pub version: Option<Version>,
}

impl TasklineMetadata {
    /// Parse metadata from file content
    pub fn parse(content: &str) -> Self {
        let mut codename = String::new();
        let mut version = None;
        
        for line in content.lines() {
            if line.starts_with("@Taskline codename ") {
                codename = line[19..].to_string();
            } else if line.starts_with("@Taskline version ") {
                if let Ok(v) = Version::parse(&line[18..]) {
                    version = Some(v);
                }
            }
        }
        
        Self { codename, version }
    }
    
    /// Generate header content
    pub fn to_header(&self) -> String {
        match &self.version {
            Some(v) => format!("@Taskline codename {}\n@Taskline version {}\n\n", self.codename, v),
            None => format!("@Taskline codename {}\n\n", self.codename),
        }
    }
}

/// Common constants
pub mod constants {
    pub const TASKLINE_EXTENSION: &str = "tskln";
    pub const DEFAULT_VERSION: &str = "v0.0.1";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(Version::parse("v1.2.3").unwrap(), Version::new(1, 2, 3));
        assert_eq!(Version::parse("v0.0.1").unwrap(), Version::new(0, 0, 1));
        assert!(Version::parse("1.2.3").is_err()); // Missing 'v'
        assert!(Version::parse("v1.2").is_err()); // Missing patch
    }

    #[test]
    fn test_version_bumping() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.bump_patch(), Version::new(1, 2, 4));
        assert_eq!(v.bump_minor(), Version::new(1, 3, 0));
        assert_eq!(v.bump_major(), Version::new(2, 0, 0));
    }

    #[test]
    fn test_metadata_parsing() {
        let content = "@Taskline codename test-script\n@Taskline version v1.2.3\n\n// Content";
        let meta = TasklineMetadata::parse(content);
        assert_eq!(meta.codename, "test-script");
        assert_eq!(meta.version, Some(Version::new(1, 2, 3)));
    }
}