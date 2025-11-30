# Admin Backend (Axum + Postgres + ClickHouse + Kafka)

## My Prompt

```
Help me to create an API server with Rust, Axum, Sqlx, PostgreSQL and ClickHouse to store data and Kafka for stream events. The idea is this API server is a backend for an Admin Portal, it also listen for events from kafka to process. The event mainly about the user and their data. This server need to have 
1. The authen and authorization in core to ensure the security. I attach some core database tables at the last.
2.This server will provide the feature to manage users. The PostgreSQL is used to store metadata, and data to run this server. But the main data such as user's data is located in Clickhouse (our main data warehouse). This feature will work like the dashboard tool such as Metabase/Superset, when query data, it will read data schema/data catalog from Postgres, then query from the main database - ClickHouse.
3. I want to design this server with Clean Architecture with Domain Driven Design(DDD) to make it easier to extend.

As said before, I have 6 database tables as bellow. My idea is I will define the resource (the API endpoint) in table permission. When a request come, I will query to check if that use is belong to the group which have permission on that route. By default, if a resouce is not defined, it meant only admin group can access it, as the first rule, they have full permission.

users id,email,first_name,last_name,passwod,salt,status,last_login,created_time,updated_time 1,admin@sample.com,admin,super,$argon2id$v=19$m=19456,t=2,p=1$YWZzZzEyZ2Zo,asdb123mnag,active,2025-11-27T11:10:09,2025-11-20T11:10:09,2025-11-27T11:10:09
2,editor@sample.com,editor,sample,$argon2id$v=19$m=19456,t=2,p=1$YWZzZzEyZ2Zo,asdb123mnag,active,2025-11-27T11:10:09,2025-11-20T11:10:09,2025-11-27T11:10:09 

group 
id,name,description,created_time,updated_time 
1,"All Users","Everyone",2025-11-20T11:10:09,2025-11-27T11:10:09 
2,"Administrators","Admin group",2025-11-20T11:10:09,2025-11-27T11:10:09 
3,"Editors","The moderators",2025-11-20T11:10:09,2025-11-27T11:10:09 
4,"Marketings",,2025-11-20T11:10:09,2025-11-27T11:10:09 
5,"Products",,2025-11-20T11:10:09,2025-11-27T11:10:09 

group_membership 
id,user_id,group_id,created_time,updated_time 
1,1,1,2025-11-20T11:10:09,2025-11-27T11:10:09 
2,1,2,2025-11-20T11:10:09,2025-11-27T11:10:09 
3,2,3,2025-11-20T11:10:09,2025-11-27T11:10:09 

permissions_group 
id,resource,group_id,perm_value,note,created_time,updated_time 
1,"/*",2,17,"Admin permission",2025-11-20T11:10:09,2025-11-27T11:10:09 
2,"/v2/users/*",3,1,"Editor can only view users",2025-11-20T11:10:09,2025-11-27T11:10:09 

permissions_type 
id,type 
1,read 
2,write 
3,read_write
4,delete 
5,delete_read
6,delete_write
7,read_write_delete

permission_methods 
id,method,perm_type 
1,"GET",1 
2,"POST",2 
3,"PUT",2 
4,"DELETE",4
```

This service is the control plane for the Admin Portal. It exposes an Axum HTTP API, enforces RBAC using the `permissions_group` table, ingests user events from Kafka, and fans queries out to ClickHouse while keeping metadata/catalog information in Postgres.

## Architecture

- **Presentation (Axum)** – Routes live under `interfaces::http`. Authentication (`middleware::auth`) validates JWTs, authorization (`middleware::permission`) checks the caller's groups against the `permissions_group` table.
- **Application layer** – `application/*` hosts orchestration services (auth, users, data catalog, permission checks). They depend only on domain traits.
- **Domain layer** – Aggregates (`domain::user`, `domain::permission`) plus repository traits.
- **Infrastructure** – Concrete adapters:
  - `postgres::repositories` for metadata, RBAC and catalog storage.
  - `clickhouse::ClickHouseUserWarehouse` to run analytical queries.
  - `kafka` module with a background consumer and an event producer.
  - `auth::password` (Argon2) and `auth::jwt` (HS256).

Shared application state (`state::AppState`) wires these components together during bootstrap.

## Running Locally

```bash
cp config/default.toml config/local.toml        # tweak secrets, DSNs
export APP_ENV=local
CARGO_TARGET_DIR=target cargo run
```

You need:

- Postgres (tables defined in `migrations/0001_init.sql`)
- ClickHouse (any cluster/database, connection set in config)
- Kafka (topic defaults to `user-events`)

Run migrations with `sqlx migrate run`.

Default admin credentials:
- email: admin@sample.com
- password: admin

## Security Model

- Users authenticate with email/password; passwords are Argon2id hashes salted per-user.
- JWTs include `groups` and are validated on every request.
- Every route resolves to a `resource` string (e.g. `/v1/users/*`). If the resource is not explicitly present in `permissions_group`, only the administrator group (ID `2`) may access it, otherwise the first matching wildcard entry controls access. HTTP methods map to permission types (`read`, `write`, `delete`) using the `permission_methods` table.

## Data Access Flow (Metabase/Superset-style)

1. Metadata authors register ClickHouse datasets in `data_catalog`.
2. When the UI requests `/v1/catalog/query`, the service fetches the stored SQL template from Postgres and executes it against ClickHouse via the warehouse adapter.
3. Result rows stream back to the Admin Portal as JSON.

## Kafka Event Processing

`kafka::spawn_consumer` starts a background task that listens to the `user-events` topic. Events (see `domain::events::UserEvent`) can be expanded later to hydrate caches or synchronize ClickHouse aggregates.

Example Kafka message:

```json
{
  "type": "UserCreated",
  "payload": {
    "user_id": 345,
    "email": "kafka@sampl.com",
    "occurred_at": "2025-11-28T22:24:21Z",
    "metadata": {
      "status": "active"
    }
  }
}
```

## Next steps

- Extend the Kafka consumer to persist denormalized views.
- Add rate limiting / audit logging in `interfaces::http`.
- Build CI/CD hooks (dockerfile, compose, smoke tests) around this crate.

