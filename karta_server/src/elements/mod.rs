use std::{path::PathBuf, time::SystemTime};

use agdb::{DbElement, DbError, DbId, DbKeyValue, DbUserValue, DbValue, QueryId, UserValue};

pub (crate) mod node;
pub (crate) mod node_path;
pub (crate) mod nodetype;
pub (crate) mod edge;
pub (crate) mod attribute;



#[derive(Debug, Clone, PartialEq)]
pub struct SysTime(SystemTime);

impl From<SysTime> for DbValue {
    fn from(time: SysTime) -> Self {
        time.0.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().into()
    }
}

impl TryFrom<DbValue> for SysTime {
    type Error = DbError;

    fn try_from(value: DbValue) -> Result<Self, Self::Error> {
        Ok(SysTime(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(value.to_u64().unwrap())))
    }
}






