# bpl-api-tx

Api for serving transactions in accordance with the Solana Pay specification and for creating promo
tokens.

## Development Notes

- https://medium.com/codemonday/access-wsl-localhost-from-lan-for-mobile-testing-8635697f008

Set up these port forwards for windows to wsl Open ports 3000,8080,8899 in windows firewall gui

open cmd in admin

`bokoup_ports.cmd`

```
@echo off
FOR /F "delims=" %%i IN ('wsl hostname -I') DO set myip=%%i
echo %myip%
netsh interface portproxy add v4tov4 listenport=3000 listenaddress=0.0.0.0 connectport=3000 connectaddress=%myip%
netsh interface portproxy add v4tov4 listenport=8080 listenaddress=0.0.0.0 connectport=8080 connectaddress=%myip%
netsh interface portproxy add v4tov4 listenport=8899 listenaddress=0.0.0.0 connectport=8899 connectaddress=%myip%
```

Delete with:

```
netsh interface portproxy delete v4tov4 listenport=8080 listenaddress=0.0.0.0
```

start validator, indexer and web application locally

check local ip address with `ipconfig` in cmd shell

solflare mobile wallet has ability to have custom url

## Check bundlr balance

This is the hardwired platform signer key for development - replace address with production key
address if not in development.

```
 bundlr balance A9KyZmfxnKhJs8eguaKj84Ru85aT1TJTZshrcMwqoBRe --host https://node1.bundlr.network -c solana
```
