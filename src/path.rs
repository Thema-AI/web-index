use std::fmt::Display;

use uuid::Uuid;

/// The path at which a record would be stored, ignoring deconfliction added to
/// allow concurrent writing.
pub struct LogicalPath {
    dir: String,
    filename: String,
    suffix: String,
}

impl LogicalPath {
    /// Construct a logical path from a dir and filename.
    pub fn new(dir: String, filename: String, suffix: String) -> Self {
        Self {
            dir,
            filename,
            suffix,
        }
    }
}

impl Display for LogicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}.{}", self.dir, self.filename, self.suffix)
    }
}

/// The path at which a record is actually stored, including deconfliction markers.
pub struct PhysicalPath {
    dir: String,
    filename: String,
    suffix: String,
    marker: String,
}

impl PhysicalPath {
    /// Construct a new physical path with a known deconfliction marker.
    pub fn new(logical: LogicalPath, marker: String) -> Self {
        Self {
            dir: logical.dir,
            filename: logical.filename,
            suffix: logical.suffix,
            marker,
        }
    }

    /// Construct a physical path with a new deconfliction marker.
    pub fn new_default(logical: LogicalPath) -> Self {
        let uuid = Uuid::new_v4();
        Self::new(logical, format!("{uuid}"))
    }
}

impl Display for PhysicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}.{}.{}",
            self.dir, self.filename, self.marker, self.suffix
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn physical_path_contains_uuid() {
        let path = PhysicalPath::new_default(LogicalPath::new(
            "dir".into(),
            "filename".into(),
            "suffix".into(),
        ));

        Uuid::parse_str(&path.marker).unwrap();
    }

    #[test]
    fn new_default_constructs_contains_new_deconfliction_marker_each_time() {
        let path1 = PhysicalPath::new_default(LogicalPath::new(
            "dir".into(),
            "filename".into(),
            "suffix".into(),
        ));
        let path2 = PhysicalPath::new_default(LogicalPath::new(
            "dir".into(),
            "filename".into(),
            "suffix".into(),
        ));

        assert_ne!(path1.marker, path2.marker);
    }

    #[test]
    fn logical_path_converts_to_correct_str() {
        let path = LogicalPath::new("dir".into(), "filename".into(), "suffix".into());

        assert_eq!(path.to_string(), "dir/filename.suffix".to_string())
    }

    #[test]
    fn physical_path_converts_to_correct_str() {
        let logical = LogicalPath::new("dir".into(), "filename".into(), "suffix".into());
        let path = PhysicalPath::new(logical, "marker".into());

        assert_eq!(path.to_string(), "dir/filename.marker.suffix".to_string())
    }
}
