use std::sync::Arc;

use crate::{
    domain::permission::{PermissionMethod, PermissionRepository},
    shared::errors::DomainError,
};

pub const ADMIN_GROUP_ID: i64 = 2;

pub struct PermissionService {
    repo: Arc<dyn PermissionRepository>,
}

impl PermissionService {
    pub fn new(repo: Arc<dyn PermissionRepository>) -> Self {
        Self { repo }
    }

    pub async fn ensure_access(
        &self,
        group_ids: &[i64],
        resource: &str,
        method: PermissionMethod,
    ) -> Result<(), DomainError> {
        let permissions = self.repo.find_permissions_for_groups(group_ids).await?;

        for perm in permissions {
            if resource_matches(&perm.resource, resource) && perm.bits.allows(method) {
                return Ok(());
            }
        }

        if group_ids.contains(&ADMIN_GROUP_ID) {
            return Ok(());
        }

        Err(DomainError::PermissionDenied)
    }
}

fn resource_matches(pattern: &str, resource: &str) -> bool {
    if pattern == "/*" {
        return true;
    }

    if let Some(stripped) = pattern.strip_suffix('*') {
        return resource.starts_with(stripped);
    }

    pattern == resource
}
