use reqwest::multipart;
use std::io::Cursor;

pub async fn sync_info() -> String {
    let client = reqwest::Client::builder().build().unwrap();
    let res = client
        .get("http://localhost:8443/info")
        .send()
        .await
        .unwrap();
    format!("sync/info: {:?}", res.text().await)
}

pub async fn sync_download() -> String {
    let client = reqwest::Client::builder().build().unwrap();
    let res = client
        .get("http://localhost:8443/fetch")
        .send()
        .await
        .unwrap();

    let mut file = std::fs::File::create("./REMOTE_test.tirra.db").unwrap();
    let mut content = Cursor::new(res.bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();

    String::from("Downloaded ?")
}

pub async fn sync_upload() -> String {
    let form = multipart::Form::new()
        .text("key", "value")
        .file("files", "test.tirra.db")
        .await
        .unwrap();

    let client = reqwest::Client::builder().build().unwrap();
    let res = client
        .post("http://localhost:8443/push")
        .multipart(form)
        .send()
        .await
        .unwrap();

    format!("sync/push: {:?}", res.text().await)
}
