use std::sync::Arc;

use crate::{
    domain::permission::{PermissionMethod, PermissionRepository},
    shared::errors::DomainError,
};
use tracing::{debug, info};

pub const ADMIN_GROUP_ID: i32 = 2;

pub struct PermissionService {
    repo: Arc<dyn PermissionRepository>,
}

impl PermissionService {
    pub fn new(repo: Arc<dyn PermissionRepository>) -> Self {
        Self { repo }
    }

    pub async fn ensure_access(
        &self,
        group_ids: &[i32],
        resource: &str,
        method: PermissionMethod,
    ) -> Result<(), DomainError> {
        debug!(
            "Checking access for {} groups to resource '{}' with method {:?}",
            group_ids.len(),
            resource,
            method
        );
        let permissions = self.repo.find_permissions_for_groups(group_ids).await?;

        for perm in permissions {
            if resource_matches(&perm.resource, resource) && perm.bits.allows(method) {
                debug!(
                    "Access granted for groups {:?} to resource '{}' via permission '{}' with method {:?}",
                    group_ids, resource, perm.resource, method
                );
                return Ok(());
            }
        }

        if group_ids.contains(&ADMIN_GROUP_ID) {
            info!("Access granted to admin group for resource '{}'", resource);
            return Ok(());
        }

        debug!(
            "Access denied for groups {:?} to resource '{}' with method {:?}",
            group_ids, resource, method
        );
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
