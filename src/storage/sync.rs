use reqwest::{multipart, StatusCode};
use std::env;
use std::io::Cursor;
use std::path::Path;

use super::db;
use super::db::TirraDbInformation;
use super::tirracrypto::TirraCrypto;

const SYNC_URL_FOR_TESTING: &str = "http://localhost:8443";
const SYNC_TMP_FILE: &str = "tirra.sync.db";

#[derive(Debug, Clone, PartialEq)]

pub enum SyncState {
    Fetched,
    Pushed,
    LinkFailure,
    DiskFailure,
    DbNotFound,
}

#[derive(Debug, Clone)]
pub enum SyncDecision {
    ChangeOrigin,
    ChangeLocal,
    Conflict,
    ChangeCommits,
}

pub async fn sync_info(id: u64) -> String {
    let client = reqwest::Client::builder().build().unwrap();
    let res = client.get(url_endpoint("info", id)).send().await.unwrap();
    format!("{:?}", res.text().await)
}

pub async fn sync_download(id: u64) -> SyncState {
    let client = reqwest::Client::builder().build().unwrap();
    let response_result = client.get(url_endpoint("fetch", id)).send().await;

    match response_result {
        Ok(response) => {
            if response.status() == StatusCode::OK {
                // write file
                let dl_db_path = origin_db_temp_file();
                let mut file = std::fs::File::create(dl_db_path).unwrap();
                let mut content = Cursor::new(response.bytes().await.unwrap());
                std::io::copy(&mut content, &mut file).unwrap();

                SyncState::Fetched
            } else {
                SyncState::DbNotFound
            }
        }
        Err(_) => SyncState::LinkFailure,
    }
}

pub async fn sync_upload(id: u64, db_location: String) -> SyncState {
    let form_result = multipart::Form::new()
        //.text("key", "value")
        .file("files", db_location)
        .await;

    let Ok(form) = form_result else {
        return SyncState::DiskFailure;
    };

    let Ok(client) = reqwest::Client::builder().build() else {
        return SyncState::DiskFailure;
    };

    let res = client
        .post(url_endpoint("push", id))
        .multipart(form)
        .send()
        .await;
    if res.is_ok() {
        return SyncState::Pushed;
    } else {
        return SyncState::LinkFailure;
    }
}

/**
 * retrieve origin db information.
 * @param local_crypto: the crypto instance used to decrypt local database file.
 */
pub fn sync_retrieve_information(local_crypto: &TirraCrypto) -> Option<TirraDbInformation> {
    let sync_crypto = local_crypto.clone_new_db_location(origin_db_temp_file().as_str());
    let info_result = db::tirra_db_information(&sync_crypto);
    match info_result {
        Ok(info) => Some(info),
        Err(_) => None,
    }
}

pub fn decision(ours: &TirraDbInformation, theirs: &TirraDbInformation) -> SyncDecision {
    if ours.local_commit == ours.origin_commit {
        return SyncDecision::ChangeLocal;
    } else {
        if ours.origin_commit == theirs.local_commit {
            return SyncDecision::ChangeOrigin;
        } else if ours.local_commit == theirs.local_commit
            && ours.origin_commit == theirs.origin_commit
        {
            return SyncDecision::ChangeCommits;
        }
        return SyncDecision::Conflict;
    }
}

pub fn info_sync_debug(info: &TirraDbInformation) {
    // local : xxxxxxxx
    // origin: yyyyyyy
    let local_commit = match &info.local_commit {
        Some(c) => db::commit_id_string(&c),
        _ => String::from("empty"),
    };
    let origin_commit = match &info.origin_commit {
        Some(c) => db::commit_id_string(&c),
        _ => String::from("empty"),
    };

    println!(
        "local : {}\norigin: {}",
        &local_commit[0..6],
        &origin_commit[0..6]
    );
}

fn url_endpoint(resource: &str, db_id: u64) -> String {
    format!("{}/{}?id={}", SYNC_URL_FOR_TESTING, resource, db_id)
}

fn origin_db_temp_file() -> String {
    let tmp_dir = env::temp_dir();
    let tmp_path = Path::new(&tmp_dir).join(SYNC_TMP_FILE);

    match tmp_path.to_str() {
        Some(tmp_file) => String::from(tmp_file),
        None => String::from(SYNC_TMP_FILE),
    }
}
