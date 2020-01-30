use sx_types::agent::AgentId;
use holochain_persistence_api::cas::content::Address;
use std::path::{Path, PathBuf};
use holochain_persistence_api::cas::content::AddressableContent;

#[derive(Clone, Debug, Shrinkwrap)]
pub struct DatabasePath(PathBuf);


impl From<(Address, AgentId)> for DatabasePath {

    fn from((addr, id): (Address, AgentId)) -> Self {
        let database_path = PathBuf::new();
        database_path.join(format!("{}", id.address()));
        database_path.join(format!("{}", addr));
        DatabasePath(database_path.into())
    }
}


impl AsRef<Path> for DatabasePath {

    fn as_ref(&self) -> &Path {
        self.as_path() 
    }
}

