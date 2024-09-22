use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl ToString for ResourceLocation {
    fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

impl FromStr for ResourceLocation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(":") {
            Some((namespace, path)) => Self::new(namespace, path),
            None => Self::new(Self::DEFAULT_NAMESPACE, s),
        }
    }
}

impl ResourceLocation {
    pub const DEFAULT_NAMESPACE: &str = "minecraft";

    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> anyhow::Result<Self> {
        let namespace = namespace.into();
        if !Self::is_valid_namespace(&namespace) {
            anyhow::bail!("Invalid namespace: {}", namespace);
        }

        let path = path.into();
        if !Self::is_valid_path(&path) {
            anyhow::bail!("Invalid path: {}", path);
        }

        Ok(Self { namespace, path })
    }

    pub fn is_valid_namespace(namespace: &str) -> bool {
        namespace.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' || c == '.'
        })
    }

    pub fn is_valid_path(path: &str) -> bool {
        path.chars().all(|c| {
            c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || c == '_'
                || c == '-'
                || c == '.'
                || c == '/'
        })
    }

    pub fn is_valid(&self) -> bool {
        Self::is_valid_namespace(&self.namespace) && Self::is_valid_path(&self.path)
    }
}
