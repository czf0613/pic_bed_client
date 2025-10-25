use super::HTTP_CLIENT;
use std::sync::OnceLock;
use types::*;

mod types;

/// 临时登录的JWT
static JWT: OnceLock<String> = OnceLock::new();

/// 把内容上传到nas，然后返回直链URL
pub async fn push_content_to_nas<T: AsRef<str>>(
    new_name: T,
    content: &[u8],
) -> Result<String, String> {
    let server_path = format!("{}/{}", NAS_PATH_BASE, new_name.as_ref());
    let (file_existence, file_size, sign) = check_file(&server_path).await?;

    if file_existence {
        if content.len() == file_size {
            // 文件存在且大小相同，姑且认为是同一个文件，直接返回下载链接
            let download_link = format!(
                "{}/d/public/pic_bed{}?sign={}",
                NAS_URL_BASE, server_path, sign
            );
            return Ok(download_link);
        } else {
            // 根据我们前面的命名规则，此时应该是极为罕见的哈希冲突
            return Err("文件哈希冲突，上传失败".to_string());
        }
    }

    // 进行上传
    upload_file(&server_path, content).await?;

    // 再次获取文件信息确保上传成功，顺便构建下载链接
    let (_, file_size, sign) = check_file(&server_path).await?;
    if file_size != content.len() {
        return Err("文件上传后大小不匹配，上传失败".to_string());
    }

    let download_link = format!(
        "{}/d/public/pic_bed{}?sign={}",
        NAS_URL_BASE, server_path, sign
    );
    return Ok(download_link);
}

/// 直接把文件的二进制内容写进去body里直接上传
async fn upload_file(server_path: &str, content: &[u8]) -> Result<(), String> {
    let url = format!("{}/api/fs/put", NAS_URL_BASE);

    let resp = HTTP_CLIENT
        .put(url)
        .header("Authorization", JWT.get().unwrap())
        .header("File-Path", server_path)
        .body(content.to_vec())
        .send()
        .await
        .map_err(|e| format!("文件上传请求发送失败: {}", e))?
        .json::<BaseResp>()
        .await
        .map_err(|e| format!("文件上传请求解析失败: {}", e))?;

    if resp.code != 200 {
        return Err(format!(
            "文件上传请求失败，状态码: {}，{}",
            resp.code, resp.message
        ));
    }

    return Ok(());
}

/// 检查文件的状态，返回是否存在、文件大小、以及sign值（用于构建下载链接）
async fn check_file(server_path: &str) -> Result<(bool, usize, String), String> {
    let url = format!("{}/api/fs/get", NAS_URL_BASE);

    let req = GetFileInfoReq {
        path: server_path.to_string(),
    };

    let resp = HTTP_CLIENT
        .post(url)
        .header("Authorization", JWT.get().unwrap())
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("获取文件信息请求发送失败: {}", e))?
        .json::<BaseResp>()
        .await
        .map_err(|e| format!("获取文件信息请求解析失败: {}", e))?;

    if resp.message.contains("not found") {
        // 文件不存在，可以新建
        return Ok((false, 0, String::new()));
    } else if resp.code != 200 {
        return Err(format!(
            "获取文件信息请求失败，状态码: {}，{}",
            resp.code, resp.message
        ));
    }

    let resp_data = serde_json::from_value::<GetFileInfoRespData>(resp.data)
        .map_err(|e| format!("获取文件信息请求解析失败: {}", e))?;

    return Ok((true, resp_data.size, resp_data.sign));
}

pub async fn nas_login() -> Result<(), String> {
    let url = format!("{}/api/auth/login", NAS_URL_BASE);

    let req = LoginReq {
        username: NAS_USER.to_string(),
        password: NAS_PASSWORD.to_string(),
    };

    let resp = HTTP_CLIENT
        .post(url)
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("NAS登录请求发送失败: {}", e))?
        .json::<BaseResp>()
        .await
        .map_err(|e| format!("NAS登录请求解析失败: {}", e))?;

    if resp.code != 200 {
        return Err(format!("NAS登录失败，状态码: {}", resp.message));
    }

    let resp_data = serde_json::from_value::<LoginRespData>(resp.data)
        .map_err(|e| format!("NAS登录请求解析失败: {}", e))?;

    JWT.set(resp_data.token)
        .map_err(|_| "Double initialization of NAS".to_string())?;
    return Ok(());
}

pub async fn nas_logout() {
    let url = format!("{}/api/auth/logout", NAS_URL_BASE);

    let _ = HTTP_CLIENT
        .get(url)
        .header("Authorization", JWT.get().unwrap())
        .send()
        .await;
}
