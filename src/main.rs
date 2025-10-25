mod uploader;

#[tokio::main]
async fn main() {
    let jobs: Vec<_> = std::env::args()
        .skip(1)
        .map(|i| uploader::upload_to_nas(i))
        .collect();

    if jobs.is_empty() {
        eprintln!("Nothing to do!");
        std::process::exit(1);
    }

    // 初始化nas登录
    if let Err(e) = uploader::nas_login().await {
        eprintln!("NAS登录失败: ");
        eprintln!("{}", e);
        std::process::exit(2);
    }

    match futures::future::try_join_all(jobs).await {
        Ok(results) => {
            println!("Upload Success:");
            for url in results {
                println!("{}", url);
            }
        }
        Err(e) => {
            println!("Failed:");
            eprintln!("{}", e);
        }
    }

    // 退出登录
    uploader::nas_logout().await;
}
