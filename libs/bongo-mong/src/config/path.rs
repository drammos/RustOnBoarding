//! Path utitilies to traverse a configuration.
use super::options::PoolPermissionType;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The representation of the tree of keys we need to traverse to get
/// to the MongoDb configuration, assuming a nested hierarchy.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct TraversalPath {
    pub id: String,
    parent: Option<Box<Self>>,
    child: Option<Box<Self>>,
}

impl TraversalPath {
    /// Create a new root value of the path.
    pub fn root(id: &str) -> Self {
        let id = id.to_string();
        Self {
            id,
            ..Default::default()
        }
    }

    /// Add child node to the path.
    pub fn add_child(&mut self, id: &str) {
        let child = Self {
            id: id.to_string(),
            parent: Some(Box::new(self.clone())),
            child: None,
        };
        self.child = Some(Box::new(child));
    }
}

impl fmt::Display for TraversalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut path = vec![self.id.as_str()];
        let mut current = self;
        while let Some(ref child) = current.child {
            path.push(child.id.as_str());
            current = child.as_ref();
        }
        let display = path.as_slice().join("_");
        write!(f, "{}", display)
    }
}

impl<'a> FromIterator<&'a str> for TraversalPath {
    /// Create a new path from an iterator of keys.
    ///
    /// Order of keys implies ancestry.
    fn from_iter<T: IntoIterator<Item = &'a str>>(keys: T) -> Self {
        let mut root = Self::root("");
        let mut current = &mut root;
        for (i, key) in keys.into_iter().enumerate() {
            if i == 0 {
                current.id = key.to_string()
            } else {
                current.add_child(key);
                current = current.child.as_mut().unwrap();
            };
        }
        root
    }
}

/// Descend to the children of the path.
pub struct TraversalPathIter<'a> {
    current: Option<&'a TraversalPath>,
}

impl<'a> From<&'a TraversalPath> for TraversalPathIter<'a> {
    fn from(path: &'a TraversalPath) -> Self {
        Self {
            current: Some(path),
        }
    }
}

impl<'a> Iterator for TraversalPathIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(path) = self.current {
            self.current = path.child.as_deref();
            Some(path.id.as_str())
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a TraversalPath {
    type Item = &'a str;
    type IntoIter = TraversalPathIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TraversalPathIter {
            current: Some(self),
        }
    }
}

impl From<&PermissionPath> for TraversalPath {
    fn from(path: &PermissionPath) -> Self {
        TraversalPath::root(path.permission().to_string().as_str())
    }
}

impl From<&CollectionPath> for TraversalPath {
    fn from(path: &CollectionPath) -> Self {
        let mut new = TraversalPath::root(path.collection());
        new.child = Some(Box::new(path.permission_path().into()));
        new
    }
}

impl From<&AppPath> for TraversalPath {
    fn from(path: &AppPath) -> Self {
        let mut new = TraversalPath::root(path.api_key());
        new.child = Some(Box::new(path.collection_path().into()));
        new
    }
}

/// A global traversal path.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct PermissionPath {
    permission: PoolPermissionType,
}

/// A traversal path to a collection.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct CollectionPath {
    collection: String,
    permission: PermissionPath,
}

/// A traversal path to an application.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct AppPath {
    api_key: String,
    collection: CollectionPath,
}

impl PermissionPath {
    /// Create new path based on the given `permission` type.
    pub fn new(permission: PoolPermissionType) -> Self {
        Self { permission }
    }

    /// Reference to the inner traversal path
    pub fn permission(&self) -> PoolPermissionType {
        self.permission
    }
}

impl fmt::Display for PermissionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path: TraversalPath = self.into();
        path.fmt(f)
    }
}

impl CollectionPath {
    /// Create new path to a collection configuration.
    pub fn new(collection: impl AsRef<str>, permission: PoolPermissionType) -> Self {
        Self {
            collection: collection.as_ref().to_string(),
            permission: PermissionPath::new(permission),
        }
    }

    /// Reference to the inner traversal path
    pub fn permission_path(&self) -> &PermissionPath {
        &self.permission
    }

    /// Convert to the inner traversal path
    pub fn collection(&self) -> &str {
        self.collection.as_str()
    }
}

impl fmt::Display for CollectionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path: TraversalPath = self.into();
        path.fmt(f)
    }
}

impl AppPath {
    /// Create new path based on the `api_key`, the `collection`,
    /// and the given `permission` type.
    pub fn new(
        api_key: impl AsRef<str>,
        collection: impl AsRef<str>,
        permission: PoolPermissionType,
    ) -> Self {
        Self {
            api_key: api_key.as_ref().to_string(),
            collection: CollectionPath::new(collection, permission),
        }
    }

    /// Reference to the inner traversal path
    pub fn collection_path(&self) -> &CollectionPath {
        &self.collection
    }

    /// Convert to the inner traversal path
    pub fn api_key(&self) -> &str {
        self.api_key.as_str()
    }
}

impl fmt::Display for AppPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path: TraversalPath = self.into();
        path.fmt(f)
    }
}

/// The possible variations of a configuration path
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum ConfigPath {
    Global(PermissionPath),
    Collection(CollectionPath),
    App(AppPath),
}

impl fmt::Display for ConfigPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global(path) => path.fmt(f),
            Self::Collection(path) => path.fmt(f),
            Self::App(path) => path.fmt(f),
        }
    }
}

impl From<PermissionPath> for ConfigPath {
    fn from(path: PermissionPath) -> Self {
        Self::Global(path)
    }
}

impl From<CollectionPath> for ConfigPath {
    fn from(path: CollectionPath) -> Self {
        Self::Collection(path)
    }
}

impl From<AppPath> for ConfigPath {
    fn from(path: AppPath) -> Self {
        Self::App(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path() {
        let path = AppPath::new("wat", "what", PoolPermissionType::Read);
        let path = ConfigPath::from(path);
        assert_eq!(path.to_string().as_str(), "wat_what_read");
    }

    #[test]
    fn app_path() {
        let path = AppPath::new("wat", "what", PoolPermissionType::Read);
        assert_eq!(path.to_string().as_str(), "wat_what_read");
    }

    #[test]
    fn traversal_path_to_string() {
        let nodes = ["wat", "else"];
        let path: TraversalPath = nodes.into_iter().collect();
        assert_eq!(path.to_string().as_str(), "wat_else");
    }

    #[test]
    fn traversal_path_into_iter() {
        let nodes = ["wat", "else"];
        let path: TraversalPath = nodes.into_iter().collect();
        let mut i = 0;
        for v in &path {
            assert_eq!(v, nodes[i]);
            i += 1;
        }
    }

    #[test]
    fn traversal_path_iter() {
        let path: TraversalPath = ["wat", "else"].into_iter().collect();
        let mut iter = TraversalPathIter::from(&path);
        assert_eq!(iter.next(), Some("wat"));
        assert_eq!(iter.next(), Some("else"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn traversal_path_collect() {
        let path: TraversalPath = ["wat", "else"].into_iter().collect();
        assert_eq!(path.id.as_str(), "wat");
        assert!(path.child.is_some());
        assert_eq!(path.child.unwrap().id.as_str(), "else");
    }

    #[test]
    fn path_from_keys_to_string() {
        let path: TraversalPath = ["one", "two", "three"].into_iter().collect();
        assert_eq!(path.to_string().as_str(), "one_two_three");

        let path: TraversalPath = ["one"].into_iter().collect();
        assert_eq!(path.to_string().as_str(), "one");
    }
}
