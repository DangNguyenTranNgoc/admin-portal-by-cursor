use async_trait::async_trait;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::shared::errors::DomainError;

bitflags! {
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
    pub struct PermissionBits: u32 {
        const READ = 0b0001;
        const WRITE = 0b0010;
        const DELETE = 0b0100;
    }
}

impl PermissionBits {
    pub fn allows(&self, method: PermissionMethod) -> bool {
        match method {
            PermissionMethod::Read => self.contains(PermissionBits::READ),
            PermissionMethod::Write => self.contains(PermissionBits::WRITE),
            PermissionMethod::Delete => self.contains(PermissionBits::DELETE),
        }
    }

    pub fn full() -> Self {
        PermissionBits::READ | PermissionBits::WRITE | PermissionBits::DELETE
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PermissionMethod {
    Read,
    Write,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub group_id: i32,
    pub bits: PermissionBits,
}

#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn find_permissions_for_groups(
        &self,
        group_ids: &[i32],
    ) -> Result<Vec<Permission>, DomainError>;
}
