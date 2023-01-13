# api-data

Includes migrations and cli to reset and apply migrations to separate Postgres as a service instance
on GCP.

## to reset schema in localnet db instance

```
cargo run -- reset-schema
```

The cli is setup to use the localnet db as the default.

## to update metadata from devnet db instance to localnet db instance

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
