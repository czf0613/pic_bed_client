use futures::StreamExt;
pub use nas_api::{nas_login, nas_logout};
use sha2::{Digest, Sha256};
use std::{ffi::OsString, path::Path, time::Duration};
use tokio::io::AsyncReadExt;

mod nas_api;

/// 上传文件到nas，可以接受http的链接或者本地的文件路径
pub async fn upload_to_nas<T: Into<String>>(any_url: T) -> Result<String, String> {
    let url: String = any_url.into();

    let content = if url.starts_with("http://") || url.starts_with("https://") {
        prepare_content_from_web(&url).await?
    } else {
        prepare_content_from_file(&url).await?
    };

    let new_name = rename_file(&url, &content);

    return nas_api::push_content_to_nas(&new_name, &content).await;
}

/// 文件大小限制100MB
const SIZE_LIMIT: u64 = 100 * 1048576;
static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    let builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0.1 Safari/605.1.15")
        .no_proxy();

    return builder.build().unwrap();
});

/// 从本地文件读取内容
async fn prepare_content_from_file(path: &str) -> Result<Vec<u8>, String> {
    match tokio::fs::File::open(path).await {
        Ok(mut file) => {
            let metadata = file
                .metadata()
                .await
                .map_err(|e| format!("获取文件元数据失败: {}", e))?;
            let file_size = metadata.len();

            if file_size > SIZE_LIMIT {
                return Err(format!(
                    "文件大小超过限制: {:.2}MB",
                    (file_size as f64) / 1048576.0
                ));
            }

            let mut buf: Vec<u8> = Vec::with_capacity(file_size as usize);
            file.read_to_end(&mut buf)
                .await
                .map_err(|e| format!("读取文件内容失败: {}", e))?;
            return Ok(buf);
        }
        Err(e) => Err(format!("打开文件失败: {}", e)),
    }
}

/// 从网络链接下载内容，但不保熟，如果这个链接提供的不是图片也没办法
async fn prepare_content_from_web(url: &str) -> Result<Vec<u8>, String> {
    let resp = HTTP_CLIENT
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求URL失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("请求URL失败，状态码: {}", resp.status()));
    }

    match resp.content_length() {
        Some(len) => {
            if len > SIZE_LIMIT {
                return Err(format!(
                    "文件大小超过限制: {:.2}MB",
                    (len as f64) / 1048576.0
                ));
            }

            // 这个内存不大，可以放心读
            return resp
                .bytes()
                .await
                .map(|b| b.to_vec())
                .map_err(|e| format!("读取响应内容失败: {}", e));
        }
        None => {
            // 无法拿到长度也不一定是错的，改成stream的方式来读取，读爆内存限制就停
            let mut buf: Vec<u8> = Vec::new();
            let mut stream = resp.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| format!("读取响应内容失败: {}", e))?;

                if chunk.len() + buf.len() > SIZE_LIMIT as usize {
                    return Err(format!(
                        "文件大小超过限制: {:.2}MB（流式读取已停止）",
                        (buf.len() as f64) / 1048576.0
                    ));
                } else {
                    buf.extend_from_slice(&chunk);
                }
            }

            return Ok(buf);
        }
    }
}

fn rename_file(old_name: &str, content: &[u8]) -> String {
    // 为了防止命名空间逃逸以及重复文件的问题（其实极其罕见）
    // 直接使用SHA-256的值作为文件名，假如服务器上已经有这个hash，并且文件大小跟现在的一样，那么服务器就会认为你已经存在了这个文件
    // 如果有这个hash，但文件大小不一致，说明出现了碰撞，服务器会报错
    let hash = Sha256::digest(content);

    // 有些名字是从URL里面尝试获取的，而且后面可能跟query params，要特殊处理一下
    let extracted_path = if old_name.starts_with("http://") || old_name.starts_with("https://") {
        // 这里可以放心unwrap，因为用户这样输入已经意味着这是一个合法的网络链接
        let url = reqwest::Url::parse(old_name).unwrap();

        // 剩下的就跟文件路径的处理是一样的了
        url.path().to_string()
    } else {
        // 从文件名里面来的，直接解析
        old_name.to_string()
    };

    let path = Path::new(&extracted_path);
    let ext_name = path
        .extension()
        .unwrap_or(&OsString::from("unknown"))
        .to_string_lossy()
        .to_string();

    return format!("{:x}.{}", hash, ext_name);
}
