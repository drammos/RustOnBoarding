//! Supported options.
use crate::error::{
    BongoError::{self, MongoDbUriCreate},
    Result,
};
use mongodb::options::ClientOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Options that do not belong to the set of strict connection options specified in [MongoDb documentation][mdb-opts].
///
/// As such, they provide flexibility in configuring features built on top
/// of this module.
///
/// [mdb-opts]: https://docs.mongodb.com/manual/reference/connection-string
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[non_exhaustive]
pub enum LooseOption {
    #[serde(rename = "collection")]
    Collection,
}

impl FromStr for LooseOption {
    type Err = BongoError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();
        match s.as_ref() {
            "collection" => Ok(LooseOption::Collection),
            _ => Err(BongoError::UnsupportedOption(s)),
        }
    }
}

/// A map of loose options and their values.
#[derive(Clone, Debug, Deserialize)]
pub struct LooseOptions(pub HashMap<LooseOption, config::Value>);

impl LooseOptions {
    /// Get the value of a loose option, if any.
    pub fn get(&self, option: &LooseOption) -> Option<&config::Value> {
        self.0.get(option)
    }
}

impl TryFrom<config::Value> for LooseOptions {
    type Error = BongoError;

    fn try_from(value: config::Value) -> Result<Self> {
        Ok(Self(
            value
                .into_table()?
                .into_iter()
                .filter_map(|(k, v)| k.parse::<LooseOption>().ok().map(|o| (o, v)))
                .collect(),
        ))
    }
}

impl TryFrom<config::Config> for LooseOptions {
    type Error = BongoError;

    fn try_from(cfg: config::Config) -> Result<Self> {
        cfg.cache.try_into()
    }
}

/// Represent strict options used for establishing connections
/// with MongoDb.
///
/// This is complementary to `LooseOptions` and can be evaluated
/// only implicitly through `BongoOptions`.
struct StrictOptions(HashMap<String, String>);

/// Helper type to implement the `TryFrom` trait.
///
/// It is used in the respective implementation in `BongoOptions`.
struct Options((LooseOptions, StrictOptions));

impl TryFrom<HashMap<String, config::Value>> for Options {
    type Error = BongoError;

    fn try_from(value: HashMap<String, config::Value>) -> Result<Self> {
        let (loose, strict): (
            HashMap<String, config::Value>,
            HashMap<String, config::Value>,
        ) = value
            .into_iter()
            .partition(|(k, _)| k.parse::<LooseOption>().is_ok());
        let loose: HashMap<LooseOption, config::Value> = loose
            .into_iter()
            .map(|(k, v)| (k.parse::<LooseOption>().unwrap(), v))
            .collect();
        let loose = LooseOptions(loose);
        let mut stricter: HashMap<String, String> = Default::default();
        for (k, v) in strict {
            stricter.insert(k, v.into_string()?);
        }
        Ok(Options((loose, StrictOptions(stricter))))
    }
}

impl TryFrom<config::Value> for Options {
    type Error = BongoError;

    fn try_from(value: config::Value) -> Result<Self> {
        Self::try_from(value.into_table()?)
    }
}

/// Options used in this library.
pub struct BongoOptions {
    uri: MongoDbUri,
    other: LooseOptions,
}

impl TryFrom<HashMap<String, config::Value>> for BongoOptions {
    type Error = BongoError;

    fn try_from(value: HashMap<String, config::Value>) -> Result<Self> {
        let Options((loose, strict)): Options = value.try_into()?;
        let uri = MongoDbUri::try_from(strict)?;
        Ok(Self { uri, other: loose })
    }
}

impl TryFrom<config::Value> for BongoOptions {
    type Error = BongoError;

    fn try_from(value: config::Value) -> Result<Self> {
        let Options((loose, strict)): Options = value.try_into()?;
        let uri = MongoDbUri::try_from(strict)?;
        Ok(Self { uri, other: loose })
    }
}

impl TryFrom<config::Config> for BongoOptions {
    type Error = BongoError;

    fn try_from(cfg: config::Config) -> Result<Self> {
        cfg.cache.try_into()
    }
}

/// Set of options including ready-to-build MongoDb client options.
pub struct BongoClientOptions {
    pub connection: ClientOptions,
    pub other: LooseOptions,
}

impl BongoClientOptions {
    /// Try to parse `self` into a `ClientOptions` value.
    pub async fn try_from_bongo_options(opts: BongoOptions) -> Result<BongoClientOptions> {
        Ok(Self {
            connection: ClientOptions::parse(opts.uri.0).await?,
            other: opts.other,
        })
    }
}

/// Pool permision types.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PoolPermissionType {
    Read,
    Write,
}

impl fmt::Display for PoolPermissionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

impl Default for PoolPermissionType {
    /// The default permission type
    fn default() -> Self {
        Self::Read
    }
}

impl PoolPermissionType {
    /// Get a lowercase string of the serialized representation of `self`.
    pub fn to_lowercase_string(&self) -> String {
        self.to_string().to_lowercase()
    }
}

impl FromStr for PoolPermissionType {
    type Err = BongoError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();
        match s.as_ref() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            _ => Err(BongoError::UnknownPermission(s)),
        }
    }
}

/// A MongoDb connection string
pub struct MongoDbUri(pub String);

impl TryFrom<StrictOptions> for MongoDbUri {
    type Error = BongoError;

    fn try_from(options: StrictOptions) -> Result<Self> {
        let base_uri = options
            .0
            .get("baseUri")
            .ok_or_else(|| MongoDbUriCreate("Base URI is not provided".into()))?;

        Ok(Self(format!(
            "{}?{}",
            base_uri,
            options
                .0
                .iter()
                .filter(|(k, _)| k.as_str() != "baseUri")
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .as_slice()
                .join("&")
        )))
    }
}

impl MongoDbUri {
    /// Try to create ready-to-build client options for MongoDb.
    pub async fn try_into_client_options(&self) -> Result<ClientOptions> {
        Ok(ClientOptions::parse(&self.0).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, File, FileFormat};

    #[test]
    fn bongo_options_ok() {
        let key_value = r#"{
            "baseUri": "mongodb://wat.com/",
            "maxPoolSize": "15",
            "connectTimeoutMS": "15",
            "collection": "wat"
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(key_value, FileFormat::Json))
            .build()
            .unwrap();
        let bongo_options: BongoOptions = config.try_into().unwrap();
        assert!(
            bongo_options.uri.0.as_str() == "mongodb://wat.com/?maxPoolSize=15&connectTimeoutMS=15"
                || bongo_options.uri.0.as_str()
                    == "mongodb://wat.com/?connectTimeoutMS=15&maxPoolSize=15"
        );
    }

    #[test]
    fn parse_permission_type() {
        assert!("read".parse::<PoolPermissionType>().is_ok());
        assert!("write".parse::<PoolPermissionType>().is_ok());
        assert!("Read".parse::<PoolPermissionType>().is_ok());
        assert!("Write".parse::<PoolPermissionType>().is_ok());
        match "reed".parse::<PoolPermissionType>() {
            Err(BongoError::UnknownPermission(msg)) => {
                assert_eq!(msg.as_str(), "reed");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_loose_option() {
        assert!("collection".parse::<LooseOption>().is_ok());
        assert!("collection ".parse::<LooseOption>().is_ok());
        assert!("   collection ".parse::<LooseOption>().is_ok());
        assert!("ColLection".parse::<LooseOption>().is_ok());
        assert!("colletion".parse::<LooseOption>().is_err());
        match "colletion".parse::<LooseOption>() {
            Err(BongoError::UnsupportedOption(msg)) => {
                assert_eq!(msg.as_str(), "colletion");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn display_pool_permission_type() {
        assert_eq!(format!("{}", PoolPermissionType::Read), "read".to_string());
        assert_eq!(
            format!("{}", PoolPermissionType::Write),
            "write".to_string()
        );
    }

    #[test]
    fn loose_options_hashmap_from_config() {
        let key_value = r#"{
            "collection": "wat",
            "maxPoolSize": "15",
            "connectTimeoutMS": "15"
        }
        "#;
        let config = Config::builder()
            .add_source(File::from_str(key_value, FileFormat::Json))
            .build()
            .unwrap();
        let loose_options = LooseOptions::try_from(config);
        let loose_options = loose_options.unwrap();
        assert_eq!(
            loose_options.get(&LooseOption::Collection),
            Some(&config::Value::new(None, "wat"))
        );
    }

    #[test]
    fn loose_options_hashmap_from_value() {
        let key_value = r#"{
            "collection": "wat",
            "maxPoolSize": "15",
            "connectTimeoutMS": "15"
        }
        "#;
        let config = Config::builder()
            .add_source(File::from_str(key_value, FileFormat::Json))
            .build()
            .unwrap();
        let loose_options = LooseOptions::try_from(config.cache);
        let loose_options = loose_options.unwrap();
        assert_eq!(
            loose_options.get(&LooseOption::Collection),
            Some(&config::Value::new(None, "wat"))
        );
    }

    #[tokio::test]
    async fn client_options_from_mongo_uri_ok() {
        let uri = MongoDbUri("mongodb://wat.com/?connectTimeoutMS=15&maxPoolSize=15".into());
        let options = uri.try_into_client_options().await;
        assert!(options.is_ok());
        let options = options.unwrap();
        assert_eq!(options.max_pool_size, Some(15));
        assert_eq!(options.connect_timeout.unwrap().as_millis(), 15);
    }

    #[tokio::test]
    async fn client_options_from_mongo_uri_err() {
        // Missing delimiter between host and options
        let uri = MongoDbUri("mongodb://wat.com?connectTimeoutMS=15&maxPoolSize=15".into());
        let options = uri.try_into_client_options().await;
        assert!(options.is_err());
        // Invalid option name
        let uri = MongoDbUri("mongodb://wat.com/?connect_timeout=15&maxPoolSize=15".into());
        let options = uri.try_into_client_options().await;
        assert!(options.is_err());
    }
}
