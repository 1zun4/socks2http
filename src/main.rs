use std::{net::SocketAddr, sync::Arc, time::SystemTime};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use log::{error, info};
use socks5_impl::{protocol::{Address, AsyncStreamOperation, AuthMethod, Reply, UserKey}, server::{AuthExecutor, ClientConnection, IncomingConnection, Server}};
use tokio::{io, net::TcpStream};
use async_http_proxy::http_connect_tokio_with_basic_auth;

const DEFAULT_VERBOSITY: &str = "info";
// keep SOCKS5 addr local - there is NO AUTHENTICATION!
const PROXY_SOCKS5_ADDR: &str = "127.0.0.1:42000";
const PROXY_HTTP_ADDR: &str = "http.yourproxyprovider.com:40000";

#[tokio::main]
async fn main() -> Result<()> {
    let default = format!("{}={}", module_path!(), DEFAULT_VERBOSITY);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default)).init();

    info!("SOCKS2HTTP v{}", env!("CARGO_PKG_VERSION"));

    socks5_loop().await?;
    Ok(())
}

/// Credentials Forwarder
struct Authenticator;

#[async_trait]
impl AuthExecutor for Authenticator {
    
    async fn execute(&self, stream: &mut TcpStream) -> Self::Output {
        use socks5_impl::protocol::password_method::{Request, Response, Status::*};
        let req = Request::retrieve_from_async_stream(stream).await?;
        let user_key = req.user_key;

        // Always return success
        let resp = Response::new(Succeeded);
        resp.write_to_async_stream(stream).await?;

        Ok(user_key)
    }

    type Output = Result<UserKey>;

    fn auth_method(&self) -> AuthMethod {
        return AuthMethod::UserPass;
    }

}

async fn socks5_loop() -> Result<()> {
    // Start SOCKS5 server with authentication
    let addr: SocketAddr = PROXY_SOCKS5_ADDR.parse()?;
    let authenticator = Arc::new(Authenticator);
    let server = Server::bind(addr, authenticator).await?;
    info!("Socks5 server listen on {}", server.local_addr()?);

    while let Ok((conn, _)) = server.accept().await {
        tokio::spawn(async move {
            if let Err(err) = handle(conn).await {
                error!("Error: {:?}", err);
            }
        });
    }

    Ok(())
}


async fn handle<S>(conn: IncomingConnection<S>) -> Result<()> 
where S: Send + Sync + 'static {
    let (conn, res) = conn.authenticate().await?;

    use as_any::AsAny;
    let user_key = if let Some(res) = res.as_any().downcast_ref::<Result<UserKey>>() {
        res.as_ref().map_err(|err| anyhow!(format!("Authenticated failed because of {:?}", err)))?.clone()
    } else {
        bail!("Unexpected authentication result: {:?}", res.as_any().type_id())
    };
    
    match conn.wait_request().await? {
        ClientConnection::UdpAssociate(associate, _) => {
            let mut conn = associate.reply(Reply::CommandNotSupported, Address::unspecified()).await?;
            conn.shutdown().await?;
        }
        ClientConnection::Bind(bind, _) => {
            let mut conn = bind.reply(Reply::CommandNotSupported, Address::unspecified()).await?;
            conn.shutdown().await?;
        }
        ClientConnection::Connect(connect, server_addr) => {
            let mut client_stream = connect.reply(Reply::Succeeded, Address::unspecified()).await?;

            info!("Connection to {}:{} via {}...", server_addr.domain(), server_addr.port(), PROXY_HTTP_ADDR);
            let start_connection_time = SystemTime::now();

            let mut tcp_stream = TcpStream::connect(PROXY_HTTP_ADDR).await?;
            let connected_http_time = start_connection_time.elapsed()?;
            
            http_connect_tokio_with_basic_auth(
                &mut tcp_stream,
                &server_addr.domain(),
                server_addr.port(),
                &user_key.username,
                &user_key.password,
            ).await?;
            let authentication_time = start_connection_time.elapsed()?;

            // Throws error if the connection is aborted, therefor the connection is closed and we don't want to handle the result
            let _ = io::copy_bidirectional(&mut client_stream, &mut tcp_stream).await;
            let data_transfer_time = start_connection_time.elapsed()?;

            info!("Connection to {}:{} closed after {}ms (HTTP: {}ms, Auth: {}ms, Data: {}ms)", 
                server_addr.domain(),
                server_addr.port(),
                data_transfer_time.as_millis(),
                connected_http_time.as_millis(),
                authentication_time.as_millis() - connected_http_time.as_millis(),
                data_transfer_time.as_millis() - connected_http_time.as_millis() - authentication_time.as_millis()
            );
        }
    }

    Ok(())
}