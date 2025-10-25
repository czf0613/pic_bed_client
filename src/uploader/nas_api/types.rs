use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(super) struct BaseResp {
    pub code: i32,
    pub message: String,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub(super) struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub(super) struct LoginRespData {
    pub token: String,
}

#[derive(Serialize)]
pub(super) struct GetFileInfoReq {
    pub path: String,
}

#[derive(Deserialize)]
pub(super) struct GetFileInfoRespData {
    pub size: usize,
    pub sign: String,
}

pub(super) const NAS_URL_BASE: &'static str = "https://nas.kevinc.ltd:30002";
pub(super) const NAS_USER: &'static str = "pic_bed";
pub(super) const NAS_PASSWORD: &'static str = "sP0bIw0m3dLBKPvCGRjZ";
pub(super) const NAS_PATH_BASE: &'static str = "/storage";
