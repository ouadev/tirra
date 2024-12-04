use std::io::Cursor;

pub async fn sync_download() -> String {
    let client = reqwest::Client::builder().build().unwrap();
    let res = client
        .get("http://localhost:8000/index.html")
        .send()
        .await
        .unwrap();

    let mut file = std::fs::File::create("/tmp/tirra.html").unwrap();
    let mut content = Cursor::new(res.bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();

    String::from("Downloaded ?")
}

pub async fn sync_upload() -> String {
    let client = reqwest::Client::builder().build().unwrap();
    let res = client
        .get("http://localhost:8000/index.html")
        .send()
        .await
        .unwrap();

    let mut file = std::fs::File::create("/tmp/tirra.html").unwrap();
    let mut content = Cursor::new(res.bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();

    String::from("Downloaded ?")
}
