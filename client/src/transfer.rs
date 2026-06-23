//! File Upload, Delete, List & Download Logic

use crate::ui;
use common::{
    hashing, protocol::ProtocolConnection, protocol::BUFFER_SIZE, FileHeader, VeriflowError,
};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

// comfy table
use comfy_table::presets::NOTHING;
use comfy_table::Table;

/// Helper function for establishing a connection
async fn connect_to_server(ip: &str) -> common::Result<ProtocolConnection> {
    println!("Connecting to {ip}...");
    
    // connect via TCP stream, intercept io::Error for connection VeriflowError 
    let stream = TcpStream::connect(ip).await.map_err(|e| VeriflowError::ConnectionFailed {
        ip: ip.to_string(),
        source: e,
    })?;

    // move ownership of stream into ProtocolConnection
    Ok(ProtocolConnection::new(stream).await?)
}

/// Upload to Server
pub async fn upload_file(path: &Path, ip: &str) -> common::Result<()> {
    // Offline Logic (Validation)

    // get file with tokio (VeriflowError if it doesn't exist)
    let mut file = File::open(path).await?;

    // get file metadata
    let file_metadata = file.metadata().await?;
    let file_size = file_metadata.len();

    // get file name -- Strict error handling (Allow ONLY UTF-8 characters)
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(VeriflowError::InvalidPath)?;

    // Hashing
    println!("Starting Hashing...");

    // create progress bar
    // set max to len of file and operation description
    let mut progress_bar = ui::create_progress_bar(file_size, "Hashing ...");

    let file_hash =
        hashing::hash_file(path, |bytes_read| progress_bar.inc(bytes_read as u64)).await?;

    // finish progress bar
    progress_bar.finish_with_message("Hashing Complete!");

    println!("File Hash: {file_hash}");

    // connect to server
    let mut connection = connect_to_server(ip).await?;

    // Setup FileHeader
    let file_header: FileHeader = FileHeader::Upload {
        name: String::from(file_name),
        size: file_size,
        hash: file_hash,
    };

    // Serialise the body
    // JSON string
    let header_json = serde_json::to_string(&file_header)?;

    // send header via helper
    connection.send_header(&header_json).await?;

    // File Upload
    println!("Starting Uploading...");

    // create progress bar
    // set max to len of file and operation description
    progress_bar = ui::create_progress_bar(file_size, "Uploading ...");

    // Stream the body

    // Buffer
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    // read file using buffer
    loop {
        // Read chunk from file (number of bytes successfully read)
        let bytes_read: usize = file.read(&mut buffer).await?;

        // finish reading file
        if bytes_read == 0 {
            // break loop
            break;
        }

        // update progress bar
        progress_bar.inc(bytes_read as u64);

        // load the chunk from file
        let current_chunk: &[u8] = &buffer[..bytes_read];

        // update stream with current chunk reference
        connection.send_data(current_chunk).await?;
    }

    // finish progress bar
    progress_bar.finish_with_message("Upload Complete!");

    // wait for server response that the file has been successfully uploaded
    println!("Waiting for server confirmation...");

    // get prefix
    let prefix_len = connection.read_prefix().await?;
    // get JSON bytes from stream
    let header: Vec<u8> = connection.read_body(prefix_len).await?;
    // convert bytes into json
    let response: FileHeader = serde_json::from_slice(&header)?;

    // Check response
    response.unpack_response()?;

    Ok(())
}

/// Download from Server
pub async fn download_file(path: &Path, ip: &str, download_dir: &Path) -> common::Result<()> {
    // connect to server
    let mut connection = connect_to_server(ip).await?;

    // get file name -- Strict error handling (Allow ONLY UTF-8 characters)
    let file_name = path
        .to_str()
        .ok_or(VeriflowError::InvalidPath)?
        .replace("\\", "/");

    // Setup FileHeader
    let file_header: FileHeader = FileHeader::Download {
        name: file_name.clone(),
    };

    // Serialise the body
    // JSON string
    let header_json = serde_json::to_string(&file_header)?;

    // send header via helper
    connection.send_header(&header_json).await?;

    println!("Waiting for server response...");

    // get prefix
    let prefix_len = connection.read_prefix().await?;
    // get JSON bytes from stream
    let header: Vec<u8> = connection.read_body(prefix_len).await?;
    // convert bytes into json
    let file_header: FileHeader = serde_json::from_slice(&header)?;

    // extract size and hash from header
    let (received_size, received_hash) = match file_header {
        FileHeader::Upload { size, hash, .. } => (size, hash),
        FileHeader::Error(e) => return Err(VeriflowError::ServerError(e)),
        other => return Err(VeriflowError::UnexpectedFileHeader(format!("{:?}", other))),
    };

    // Downloading to disk

    // // Ensure download dir exists
    // tokio::fs::create_dir_all(download_dir).await?;

    // combine into a single valid path
    let full_download_path = download_dir.join(&file_name);

    // make sure that the subdirectory exists before creating the file
    if let Some(parent_dir) = full_download_path.parent() {
        tokio::fs::create_dir_all(parent_dir).await?;
    }

    // create file on disk
    let mut download_file = File::create(&full_download_path).await?;

    // create progress bar (download)
    // set max to len of file and operation description
    let progress_bar = ui::create_progress_bar(received_size, "Downloading ...");

    connection
        .read_file_to_disk_with_pb(&mut download_file, received_size, |bytes_read| {
            progress_bar.inc(bytes_read as u64);
        })
        .await?;

    progress_bar.finish_with_message("Download Complete!");

    // Verification (Hashing)
    println!("Verifying File Integrity...");

    // create progress bar
    // set max to len of file and operation description
    let progress_bar = ui::create_progress_bar(received_size, "Hashing ...");

    let file_hash = hashing::hash_file(&full_download_path, |bytes_read| {
        progress_bar.inc(bytes_read as u64)
    })
    .await?;

    // finish progress bar
    progress_bar.finish_with_message("Hashing Complete!");

    // check if hash is not the same
    if file_hash != received_hash {
        // clean up the corrupted file
        tokio::fs::remove_file(&full_download_path).await?;
        println!("File removed!");

        // return error
        return Err(VeriflowError::HashMismatch);
    }

    Ok(())
}

/// Delete from Server
pub async fn delete_file(path: &Path, ip: &str) -> common::Result<()> {
    // connect to server
    let mut connection = connect_to_server(ip).await?;

    // get file name -- Strict error handling (Allow ONLY UTF-8 characters)
    let file_name = path
        .to_str()
        .ok_or(VeriflowError::InvalidPath)?
        .replace("\\", "/");

    // Setup FileHeader
    let file_header: FileHeader = FileHeader::Delete {
        name: file_name.clone(),
    };

    // Serialise the body
    // JSON string
    let header_json = serde_json::to_string(&file_header)?;

    println!("Sending delete request to {ip}...");

    // send header via helper
    connection.send_header(&header_json).await?;

    // wait for server response
    // get prefix
    let prefix_len = connection.read_prefix().await?;
    // get JSON bytes from stream
    let header: Vec<u8> = connection.read_body(prefix_len).await?;
    // convert bytes into json
    let response: FileHeader = serde_json::from_slice(&header)?;

    // Check response
    response.unpack_response()?;

    Ok(())
}

/// List Server Files
pub async fn list_files(ip: &str) -> common::Result<()> {
    // connect to server
    let mut connection = connect_to_server(ip).await?;

    // Setup FileHeader
    let file_header: FileHeader = FileHeader::List;

    // Serialise the body
    // JSON string
    let header_json = serde_json::to_string(&file_header)?;

    println!("Sending list request to {ip}...");

    // send header via helper
    connection.send_header(&header_json).await?;

    // wait for server response
    // get prefix
    let prefix_len = connection.read_prefix().await?;
    // get JSON bytes from stream
    let header: Vec<u8> = connection.read_body(prefix_len).await?;
    // convert bytes into json
    let file_header: FileHeader = serde_json::from_slice(&header)?;

    // get size from enum
    let received_size = match file_header {
        FileHeader::Upload { size, .. } => size as usize,
        FileHeader::Error(e) => return Err(VeriflowError::ServerError(e)),
        other => return Err(VeriflowError::UnexpectedFileHeader(format!("{:?}", other))),
    };

    // read payload (one-shot)
    let payload_bytes = connection.read_payload(received_size).await?;

    // deserialise into Vec<String>
    let path_list: Vec<String> = serde_json::from_slice(&payload_bytes)?;

    // output file tree
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_header(vec!["#", "Type", "Path"]);

    // manual counter
    let mut display_id = 1;

    for path in &path_list {
        let is_dir = path.ends_with("/");

        // filter for empty directories (no other path in the list)
        if is_dir {
            let has_children = path_list
                .iter()
                .any(|other| other != path && other.starts_with(path));

            if has_children {
                continue; // skip directories that contain anything
            }
        }

        let type_of = if is_dir { "DIR" } else { "FILE" };

        table.add_row(vec![
            display_id.to_string(),
            type_of.to_string(),
            path.to_string(),
        ]);

        display_id += 1;
    }

    println!("\n{table}\n");

    Ok(())
}
