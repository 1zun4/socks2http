## SOCKS2HTTP

This project is a SOCKS5 to HTTP proxy server written in Rust. It allows clients to connect to the internet through an HTTP proxy server using SOCKS5 protocol.

### Features

* SOCKS5 server with user/password authentication (always succeeds - see note below)
* Forwards traffic to a configured HTTP proxy server
* Logs connection details and timings (connection establishment, authentication, data transfer)

**Note:** Currently, authentication on the SOCKS5 server always succeeds. This is to make forwarding credentials to HTTP proxy possible.

### Usage

**Requirements:**
* Rust compiler ([https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install))

**Building:**

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/socks2http.git
   ```
2. Navigate to the project directory:
   ```bash
   cd socks2http
   ```
3. Build the project:
   ```bash
   cargo build
   ```

**Running:**

1. Run the server:
   ```bash
   cargo run
   ```

**Client Usage:**

You can use any SOCKS5 client to connect to the server. Here's an example using `curl`:

```bash
curl -x socks5://127.0.0.1:42000 -U "username_of_http:password_of_http" https://ipinfo.io/json
```

This command will connect to `https://ipinfo.io/json` through the SOCKS5 server running on `localhost:42000`, using the provided username and password for HTTP proxy authentication.

**Configuration**

* The SOCKS5 server listens on `127.0.0.1:42000` by default. You can change this by modifying the `PROXY_SOCKS5_ADDR` constant in the source code.
* The HTTP proxy server address is configured in the `PROXY_HTTP_ADDR` constant. You need to replace it with the actual address of your HTTP proxy server.

**Logging**

The project uses the `env_logger` crate for logging. You can adjust the verbosity level by setting the `RUST_LOG` environment variable before running the server. For example:

```bash
RUST_LOG=info cargo run
```

This will only log informational messages.

### Contributing

We welcome contributions to this project! Please use common sense when contributing.

### License

This project is licensed under the GNU General Public License v3.0. A copy of the license is available in the LICENSE: LICENSE file.
