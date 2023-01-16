# api-data

Includes migrations and cli to reset and apply migrations to separate Postgres as a service instance
on GCP.

## to reset schema in localnet db instance

```
cargo run -- reset-schema
```

The cli is setup to use the localnet db as the default.

## to update metadata FROM DEVNET db instance TO LOCALNET db instance

1. change working directory to localnet
   ```
   cd localnet
   ```
2. change endpoint in `localnet/config.yaml` to devnet endpoint: `https://cool-tapir-17.hasura.app`
3. export devnet metadata overwriting metadata in `localnet` directory using the secrets from
   `devnet/.env`
   ```
   hasura metadata export --envfile ../devnet/.env
   ```
4. change endpoint in `localnet/config.yaml` back to localnet endpoint:
   `https://shining-sailfish-15.hasura.app`
5. apply metadata to localnet instance
   ```
   hasura metadata apply
   ```

## to update metadata FROM LOCALNET db instance TO DEVNET db instance

1. apply migrations
   ```
   cd api-data
   cargo run -- --db-url devnet apply-migrations
   ```
2. change working directory to devnet
   ```
   cd metadata/devnet
   ```
3. change endpoint in `devnet/config.yaml` to localnet endpoint:
   `https://shining-sailfish-15.hasura.app`
4. export localnet metadata overwriting metadata in `devnet` directory using the secrets from
   `localnet/.env`
   ```
   hasura metadata export --envfile ../localnet/.env
   ```
5. change endpoint in `devnet/config.yaml` back to devnet endpoint:
   `https://cool-tapir-17.hasura.app`
6. apply metadata to localnet instance
   ```
   hasura metadata apply
   ```
