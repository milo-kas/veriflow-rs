use crate::{Result, VeriflowError};
use std::cmp;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// convention: 4096B or 8192B
// Buffer size of 8kb for TCP
pub const BUFFER_SIZE: usize = 4096;

// Header size (max 4kb)
pub const MAX_HEADER_SIZE: usize = 4096;

// Max one-shot payload (max 10mb)
pub const MAX_PAYLOAD_SIZE: usize = 10485760;

///Represents the custom Protocol read and send methods built on top of Tcp
pub struct ProtocolConnection {
    stream: TcpStream,
}

impl ProtocolConnection {
    /// Creates a new protocol connection
    ///
    /// # Arguments
    /// * 'stream' - takes in a 'TcpStream' to base our protocol connection on
    ///
    /// # Returns
    /// A new custom protocol connection
    /// # Examples
    ///
    /// ```
    /// async fn some_func() -> Result<(), common::VeriflowError> {
    ///     use common::protocol::ProtocolConnection;
    ///     use tokio::net::TcpStream;
    ///     let stream = TcpStream::connect("127.0.0.1").await?;
    ///     let connection = ProtocolConnection::new(stream).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(stream: TcpStream) -> Result<ProtocolConnection> {
        //returns a new connection object
        Ok(ProtocolConnection { stream })
    }

    /// Sends the custom json header
    ///
    /// # Arguments
    /// * 'header' - A '&str' that contains the serialized josn
    ///
    /// # Returns
    /// A 'bool' value to represent if the send function succeeded
    pub async fn send_header(&mut self, header: &str) -> Result<()> {
        //turns data into bytes and gets the length
        let data_as_bytes = header.as_bytes();
        let data_byte_len = data_as_bytes.len() as u32;

        // send length prefix
        // convert u32 to big-endian bytes
        self.send_data(&data_byte_len.to_be_bytes()).await?;

        // Send as json body
        self.send_data(data_as_bytes).await?;

        // Flush to ensure the bytes actually leave the network buffer
        self.stream.flush().await?;

        Ok(())
    }
    /// Sending function designed to send data based on a buffer
    /// # Arguments
    /// * 'buffer' - a '&[u8]' which contains the data to be sent in byte format
    ///  
    /// # Returns
    /// A generic Result indicating success or failure
    pub async fn send_data(&mut self, buffer: &[u8]) -> Result<()> {
        self.stream.write_all(buffer).await?;
        Ok(())
    }

    ///Reads the prefixed length of the header
    ///
    /// #Returns
    /// A 'Result' of usize representing the size of the incoming header
    pub async fn read_prefix(&mut self) -> Result<usize> {
        //creates the prefix buffer
        let mut buf: [u8; 4] = [0u8; 4];
        self.stream.read_exact(&mut buf).await?;

        Ok(u32::from_be_bytes(buf) as usize)
    }

    ///Reads a number of bytes specified
    /// # Arguments
    /// * 'buffer_len' - Represents the number of bytes to read
    ///
    /// # Returns
    /// A 'Result' of 'Vec<u8>' which is the data received through the connection stream
    pub async fn read_body(&mut self, buffer_len: usize) -> Result<Vec<u8>> {
        // Verifies max header size before creating buffer
        if buffer_len > MAX_HEADER_SIZE {
            return Err(VeriflowError::HeaderSizeExceeded(buffer_len));
        }

        //creates a buffer for a custom size
        let mut buf = vec![0u8; buffer_len];

        self.stream.read_exact(&mut buf).await?;

        Ok(buf)
    }

    /// Read entire payload into memory
    pub async fn read_payload(&mut self, payload_len: usize) -> Result<Vec<u8>> {
        if payload_len > MAX_PAYLOAD_SIZE {
            return Err(VeriflowError::PayloadSizeExceeded(payload_len));
        }

        // Read payload (one-shot)
        let mut buf = vec![0u8; payload_len];
        self.stream.read_exact(&mut buf).await?;

        Ok(buf)
    }

    pub async fn write_file_to_stream(&mut self, input: &mut File, file_size: u64) -> Result<()> {
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut total_bytes_read: u64 = 0;

        loop {
            if total_bytes_read >= file_size {
                break;
            }
            let remaining_bytes = file_size - total_bytes_read;
            let bytes_to_read: usize = cmp::min(buffer.len() as u64, remaining_bytes) as usize;
            input.read_exact(&mut buffer[..bytes_to_read]).await?;
            self.stream.write_all(&buffer[..bytes_to_read]).await?;
            total_bytes_read += bytes_to_read as u64;
        }
        self.stream.flush().await?;
        Ok(())
    }

    /// Streams a file to disk from the network
    pub async fn read_file_to_disk(&mut self, output: &mut File, file_size: u64) -> Result<()> {
        // Buffer
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut total_bytes_read: u64 = 0;

        // read file using buffer
        loop {
            // check if finished reading the expected file size
            if total_bytes_read >= file_size {
                break;
            }

            // remaining bytes
            let remaining_bytes: u64 = file_size - total_bytes_read;

            // determine how much is left to read
            let bytes_to_read: usize = cmp::min(buffer.len() as u64, remaining_bytes) as usize;

            // read the chunk from buffer
            self.stream.read_exact(&mut buffer[..bytes_to_read]).await?;
            output.write_all(&buffer[..bytes_to_read]).await?;

            total_bytes_read += bytes_to_read as u64;
        }

        // flush to make sure that the data is physically written to disk
        output.flush().await?;

        Ok(())
    }


    /// Streams a file to disk from the network
    pub async fn read_file_to_disk_with_pb<F>(&mut self, output: &mut File, file_size: u64, mut progress_callback: F) -> Result<()> 
    where
        F: FnMut(usize),
    {
        // Buffer
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut total_bytes_read: u64 = 0;

        // read file using buffer
        loop {
            // check if finished reading the expected file size
            if total_bytes_read >= file_size {
                break;
            }

            // remaining bytes
            let remaining_bytes: u64 = file_size - total_bytes_read;

            // determine how much is left to read
            let bytes_to_read: usize = cmp::min(buffer.len() as u64, remaining_bytes) as usize;

            // read the chunk from buffer
            self.stream.read_exact(&mut buffer[..bytes_to_read]).await?;
            output.write_all(&buffer[..bytes_to_read]).await?;
            
            // progress bar
            progress_callback(bytes_to_read);

            total_bytes_read += bytes_to_read as u64;
        }

        // flush to make sure that the data is physically written to disk
        output.flush().await?;

        Ok(())
    }
}
