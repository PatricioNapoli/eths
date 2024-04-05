use csv::Reader;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid node file")]
    InvalidNodeFile,
    #[error("csv error: {0}")]
    CsvError(#[from] csv::Error),
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
pub struct Node {
    pub id: String,
    pub ip: String,
    pub port: u16,
}

/// Client configuration.
///
/// Nodes are read from a CSV file with the following format:
///
/// ```csv
/// id,ip,port
/// ```
///
/// CSV contains no header.
///
#[derive(Clone, Debug)]
pub struct Config {
    pub nodes: Vec<Node>,
    pub timeout: u64,
}

pub const DEFAULT_FILENAME: &str = "nodes.csv";
pub const DEFAULT_TIMEOUT: u64 = 2500;

impl Config {
    /// Parse a CSV file into a `Config`.
    pub fn from_file(file: &str) -> Result<Self, ConfigError> {
        let file_rdr = csv::Reader::from_path(file)?;
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(file_rdr.get_ref());

        Self::read_records(&mut rdr)
    }

    /// Parse a CSV string into a `Config`.
    /// Currently only used for client module test.
    #[allow(dead_code)]
    pub fn from_str(str: &str) -> Result<Self, ConfigError> {
        let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(str.as_bytes());

        Self::read_records(&mut rdr)
    }

    fn read_records<T: std::io::Read>(rdr: &mut Reader<T>) -> Result<Self, ConfigError> {
        let mut nodes = Vec::new();
        let iter = rdr.deserialize();

        for result in iter {
            let record: Node = result?;
            nodes.push(record);
        }

        if nodes.is_empty() {
            return Err(ConfigError::InvalidNodeFile);
        }

        Ok(Self {
            nodes,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    pub fn with_timeout(self, timeout: u64) -> Self {
        Self {
            timeout,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn test_config_from_file() {
        let config = Config::from_file("nodes.csv").unwrap();
        assert_eq!(config.nodes.is_empty(), false);
    }

    #[test]
    fn test_config_from_str() {
        let csv = concat!("1,2,3\n", "4,5,6",);
        let config = Config::from_str(csv).unwrap();
        assert_eq!(config.nodes.len(), 2);
        let n = &config.nodes[0];
        assert_eq!(n.id, "1");
        assert_eq!(n.ip, "2");
        assert_eq!(n.port, 3);
    }
}
