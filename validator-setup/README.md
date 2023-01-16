# validator setup

## devnet validator on edgevana

Core scripts come from [Edgevana](https://github.com/shiraz-edgevana/solana).

### initial setup

1. Setup core server
2. Install rust toolchain
3. Build geyser-plugin
4. Install nats-server
5. Create nats-server service
6. Build indexer
7. Create indexer service
8. Create script to launch validator with plugin

Need to install build-essentials pkg-config libssl-dev

### update

1. Pull `bokoup-program-library` and `geyser-plugin-nats` repos
2. `cargo-build --release` in each
3. `sudo cp /home/ubuntu/bokoup-program-library/target/release/libbpl_indexer.so /usr/local/bin`
4. `sudo cp /home/ubuntu/geyser-plugin-nats/target/release/libgeyser_plugin_nats.so /home/sol`

If there were changes to config.json 5.
`cp /home/ubuntu/geyser-plugin-nats/config.json /home/sol` 6. Update `config.json` to point to the
correct location for `libgeyser_plugin_nats.so`

Restart services 7. Indexer

```
sudo systemctl restart bpl-indexer
```

7. Validator

```
sudo systemctl restart sol
```

No new snapshots are downloaded if ledger is up to date, but take 5 to 10 minutes to reload ledger
and catch back up.
