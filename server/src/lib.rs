use std::path::PathBuf;

use serde::{Deserialize, Serialize};
pub mod server;

pub const FILE_PATH: &str = "../Veriflow/resources/";
pub const CONFIG_PATH: &str = "./config.toml";
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub network: Network,
    pub directory: Directory,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Network {
    pub ip: String,
    pub port: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Directory {
    pub path: PathBuf,
}
#[cfg(test)]
mod test {
    use std::path::PathBuf;

use crate::FILE_PATH;
use crate::server::Listener;
    pub use common::protocol::ProtocolConnection;
    pub use common::FileHeader;
    use tokio::net::TcpStream;
    #[tokio::test]
    async fn test_protocol_read_and_write(
    ) -> common::Result<()> {
        //made to avoid veriflow error
        
        //creates a server
        let mut listener = Listener::new("127.0.0.1", "0").await?;
        let addr = listener.local_addr()?;
        let server_task: tokio::task::JoinHandle<common::Result<()>> = tokio::spawn(async move {
            let stream = listener.accept_once().await?;
            let mut conn = ProtocolConnection::new(stream).await?;

            let len = conn.read_prefix().await?;
            let body = conn.read_body(len).await?;
            conn.send_header(&String::from_utf8_lossy(&body)).await?;

            Ok(())
        });
        let stream = TcpStream::connect(addr).await?;
        let mut connection = ProtocolConnection::new(stream).await?;

        // set file name
        let file_name: &str = "img.png";

        // instantiate file header
        let original_file_header = FileHeader::Upload {
            name: String::from(file_name),
            size: 4001,
            hash: String::from("abc123def"),
        };
        // serialise to JSON (Struct -> String)
        let json_string_wrapped = serde_json::to_string(&original_file_header);
        let json_string = json_string_wrapped.unwrap();
        let _result = connection.send_header(&json_string).await?;
        let header_length = connection.read_prefix().await?;
        let byte_header = connection.read_body(header_length).await?;
        let header = String::from_utf8_lossy(&byte_header);
        assert_eq!(json_string, header);
        match server_task.await {
            Ok(res) => res?,
            Err(e) => panic!("server task panicked: {e}"),
        }
        Ok(())
    }

    async fn test_server_upload() -> common::Result<()> {
        /// Test of the server upload functionality
        let mut listener = Listener::new("127.0.0.1", "0").await?;
        let addr = listener.local_addr()?;
        let server_task: tokio::task::JoinHandle<common::Result<()>> = tokio::spawn(async move {
        let stream = listener.accept_once().await?;
            let mut conn = ProtocolConnection::new(stream).await?;

            Listener::handle_client(conn, addr, PathBuf::from(FILE_PATH)).await?;

            Ok(())
        }); 
        let stream = TcpStream::connect(addr).await?;
        let mut connection = ProtocolConnection::new(stream).await?;
        let file_path = "./test_files/images.jfif";

        Ok(())
    }
}