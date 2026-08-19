use semver::Version;

pub const SNAPSHOT_PARSER_VER: Version = Version::new(0, 1, 0);
pub const SMALL_ID_LEN: usize = 8;
pub const MANIFEST_FILE: &str = "manifest.json";
pub const FILENAME_METADATA_SEPARATOR: &str = "-x";
pub const SNAPSHOT_FILE_EXTENSION: &str = ".tar.gz";
