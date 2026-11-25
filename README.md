# ihost

Share a folder over HTTP: on this machine, or with your phone and other devices on the same Wi-Fi. One command, nothing to install on the other device, only a browser.

```sh
ihost -H ~/Photos
```

```
  iHost v0.8.1 · powered by Khaled Ghallab

  Folder    /home/me/Photos
  Local     http://127.0.0.1:8000
  Network   http://192.168.1.23:8000

  (QR code)

  Started 2026-11-26 00:01:06 · press Ctrl+C to stop
```

Scan the QR code with your phone's camera to open the **Network** link, then browse and download the files. Settings you change from their defaults (upload, password, HTTPS, …) are listed below the QR code.

## What it does

- **Share files with your phone or another computer.** `-H` makes the folder reachable from devices on the same network; add `-u` to let them upload files back.
- **Preview a built website.** Serve a `dist/` folder, including single-page apps via `--try-file`, and check it on a real phone.
- **Share safely.** It listens on this machine only (`127.0.0.1`) unless you pass `-H`, and `-a user:pass` asks visitors for a password.

Also: folder listings with sorting, `index.html` rendering, resumable downloads and video seeking (Range requests), HTTP caching (`ETag` / `Last-Modified`), `gzip` / `deflate` compression, CORS / COOP / COEP headers, and optional HTTPS.

## Installation

From the project folder:

```sh
cargo install --path .
```

With HTTPS support (PKCS#12 certificates):

```sh
cargo install --path . --features tls
```

## Usage

```text
Share a folder over HTTP: on this machine, or with your phone and other devices on the same network.

Usage: ihost [OPTIONS] [FOLDER]

Arguments:
  [FOLDER]  Folder to serve [default: current folder]

Options:
  -h, --help     Print help
  -V, --version  Print version

Network:
  -H, --host         Share with other devices on the network (binds 0.0.0.0, prints the network URL and a QR code)
  -p, --port <PORT>  Port to listen on [default: 8000]
      --ip <IP>      Bind to a specific IP address instead [default: 127.0.0.1]

Files:
  -i, --index            Show index.html / index.htm instead of the folder listing when present
      --try-file <PATH>  Serve this file for missing paths, e.g. a single-page app's index.html. Relative paths are resolved against the served folder [aliases: --try-file-404]
      --nosort           Disable sorting of folder listings (by name, modified, size)
  -b, --base-url <PATH>  URL prefix used in listings and upload redirects, for running behind a reverse proxy [default: /]
      --redirect <URL>   Redirect every request to this URL (HTTP 301)

Uploads:
  -u, --upload                    Allow uploading files from the browser (several at once, CSRF-protected)
  -l, --upload-size-limit <SIZE>  Maximum upload size, in bytes or with a K/M/G/T suffix (powers of 1024), e.g. 30K, 50M, 1G [default: 8M]
      --csrf <TOKEN>              Use a fixed CSRF token instead of a random one. Unsafe: the token shows up in process listings and shell history

Security:
  -a, --auth <USER:PASS>  Require a username and password (HTTP Basic Auth)

HTTPS (requires the tls build feature):
      --cert <FILE>          Serve over HTTPS with this certificate (PKCS#12 / .p12)
      --certpass <PASSWORD>  Password for the certificate file

Response headers:
      --cors  Allow requests from other origins (Access-Control-Allow-Origin: *)
      --coop  Send Cross-Origin-Opener-Policy: same-origin
      --coep  Send Cross-Origin-Embedder-Policy: require-corp

Performance:
  -c, --compress <EXT>...  Compress files with these extensions (gzip/deflate), e.g. -c=js,d.ts. Skipped for partial (range) requests
      --nocache            Disable HTTP caching
      --norange            Disable partial downloads (Range requests), which resume and video seeking rely on
  -t, --threads <NUM>      Number of worker threads [default: 3]

Output:
  -o, --open    Open the page in the default browser
  -s, --silent  Print nothing

Examples:
  ihost                                   Serve the current folder on this machine
  ihost -H ~/Photos                       Share a folder with your phone (same Wi-Fi)
  ihost -H -u .                           ...and let other devices upload files to it
  ihost -H -a me:secret .                 Ask visitors for a username and password
  ihost -i --try-file dist/index.html dist
                                          Preview a built website (single-page app)
```

## Examples

Share a folder with your phone:

```sh
ihost -H ~/Photos
```

Let other devices upload files, with a 50 MB limit:

```sh
ihost -H -u -l 50M .
```

Require a username and password:

```sh
ihost -H -a me:secret .
```

Preview a built single-page app:

```sh
ihost -i --try-file dist/index.html dist
```

Run behind a reverse proxy that strips `/static/` before forwarding:

```sh
ihost --base-url /static/ .
```

## HTTPS Example

`--cert` expects a PKCS#12 file such as `.p12` or `.pfx`. PEM files are not accepted directly.

### 1. Generate a local certificate and `.p12`

```sh
mkdir -p /tmp/ihost-tls
cd /tmp/ihost-tls

openssl req \
  -x509 \
  -newkey rsa:2048 \
  -nodes \
  -sha256 \
  -keyout key.pem \
  -out cert.pem \
  -days 7 \
  -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost"

openssl pkcs12 \
  -export \
  -inkey key.pem \
  -in cert.pem \
  -out localhost.p12 \
  -passout pass:changeit
```

### 2. Start the server

```sh
ihost \
  -i \
  --ip 127.0.0.1 \
  --port 8443 \
  --cert /tmp/ihost-tls/localhost.p12 \
  --certpass changeit \
  .
```

### 3. Verify it

Strict verification:

```sh
curl --fail --silent --show-error \
  --cacert /tmp/ihost-tls/cert.pem \
  https://localhost:8443/
```

Quick local smoke test:

```sh
curl -kI https://localhost:8443/
```

## License

MIT. See [LICENSE](LICENSE). ihost is based on [simple-http-server](https://github.com/TheWaWaR/simple-http-server) by Qian LinFeng.
