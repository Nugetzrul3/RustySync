use std::error::Error;
use crate::shared::{
    models::FileRow,
    utils
};
use reqwest::multipart;
use crate::shared::models::ErrorResponse;
use crate::client::apis::auth;

pub async fn delete_file(path: String) -> Result<(), Box<dyn Error>> {
    // To delete file
    let url = utils::load_url().await?;
    let client = reqwest::Client::new();
    let (mut access_token, mut expires_at) = utils::load_access_token().await?;

    // token expired, call refresh then call api
    if utils::check_expiry_time(expires_at) {
        auth::refresh_user().await?;
        (access_token, expires_at) = utils::load_access_token().await?;
    }

    let delete_req = client.delete(
        format!("{}/file/delete?path={}", url, path)
    ).bearer_auth(&access_token)
        .send().await?;


    if !delete_req.status().is_success() {
        let data = delete_req.json::<ErrorResponse>().await?;
        eprintln!("Failed to delete file: {:?}", data.error);
        return Err(Box::from(data.error));
    }

    Ok(())
}

pub async fn upload_files(files: Vec<FileRow>) -> Result<(), Box<dyn Error>> {
    // Upload created/modified files
    let url = utils::load_url().await?;
    let client = reqwest::Client::new();
    let files_form = build_file_form(files).await?;
    let (mut access_token, mut expires_at) = utils::load_access_token().await?;

    if utils::check_expiry_time(expires_at) {
        auth::refresh_user().await?;
        (access_token, expires_at) = utils::load_access_token().await?;
    }

    let upload_req = client.post(
        format!("{}/file/upload", url)
    )
        .bearer_auth(&access_token)
        .multipart(files_form)
        .send().await?;

    println!("{}", upload_req.text().await?);

    // if !upload_req.status().is_success() {
    //     let data = upload_req.json::<ErrorResponse>().await?;
    //     eprintln!("Failed to upload files: {:?}", data.error);
    //     return Err(Box::from(data.error));
    // }


    Ok(())
}

// pub async fn get_files() -> Result<Vec<FileRow>, Box<dyn Error>> {
//     // return list of all metadata for files belonging to that user
//     Ok(Vec::<FileRow>::new())
// }
//
// pub async fn get_file(path: String) -> Result<FileRow, Box<dyn Error>> {
//     // Get specific file metadata
//     Ok(FileRow::new(String::new(), String::new(), Utc::now()))
// }


// Utility functions for file uploads

// Build file multipart form
async fn build_file_form(files: Vec<FileRow>) -> Result<multipart::Form, Box<dyn Error>> {
    let mut form = multipart::Form::new();

    for file in files {
        let (filename, path) = extract_filename_filepath(&file.path().to_string());
        let file_part = multipart::Part::file(file.path()).await?;
        let last_modified_part = multipart::Part::text(file.last_modified().to_rfc3339());
        let path_part = multipart::Part::text(path);
        form = form
            .part(format!("last_modified_{}", filename), last_modified_part)
            .part(format!("path_{}", filename), path_part)
            .part(format!("file_{}", filename), file_part);
    }
    Ok(form)

}

// Extract filename and filepath from filerow object
fn extract_filename_filepath(full_path: &String) -> (String, String) {
    let mut path_tokens = full_path.split("/").collect::<Vec<&str>>();
    let file_name = path_tokens.pop().unwrap();
    let path = path_tokens.join("/");

    (file_name.to_string(), path)
}
