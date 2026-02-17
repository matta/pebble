use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    #[serde(rename = "sync-branch")]
    pub sync_branch: Option<String>,
    
    #[serde(rename = "issue-prefix")]
    pub issue_prefix: Option<String>,

    #[serde(rename = "no-db")]
    pub no_db: Option<bool>,

    #[serde(rename = "no-daemon")]
    pub no_daemon: Option<bool>,

    #[serde(rename = "auto-start-daemon")]
    pub auto_start_daemon: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mydoo_config() {
        // Read the actual config file from the workspace
        let content = std::fs::read_to_string("../mydoo/.beads/config.yaml")
            .expect("Failed to read config file");
        
        let config: Config = serde_yaml::from_str(&content)
            .expect("Failed to parse config");
            
        assert_eq!(config.sync_branch, Some("beads-sync".to_string()));
        // Verify other fields from comments or default values if they were present in the file
        // The provided file has most things commented out, so they should be None
        assert_eq!(config.issue_prefix, None);
        assert_eq!(config.no_db, None);
    }
}
