use std::{collections::HashMap, error::Error};

use anyhow::format_err;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;
use tissue_rs::TissueRequester;

pub static RE_CHECKINS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"通算回数:\s*([\d,]+)\s*回"#).expect("Invalid regex"));
pub static RE_INTERVAL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"([\d]+)日\s*([\d]+)時間\s*([\d]+)分経過"#).expect("Invalid regex"));

pub struct SurfRequester;

#[async_trait]
impl TissueRequester for SurfRequester {
    async fn get(
        &mut self,
        url: String,
        headers: HashMap<String, String>,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut builder = surf::get(url);
        for (h, v) in headers {
            builder = builder.header(&h[..], &v[..]);
        }

        let json = builder.await?.body_json().await;
        match json {
            Ok(v) => Ok(v),
            Err(e) => Err(format_err!("Failed to request Tissue: {}", e).into()),
        }
    }

    async fn post(
        &mut self,
        url: String,
        headers: HashMap<String, String>,
        body: serde_json::Value,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let mut builder = surf::post(url).body(body);
        for (h, v) in headers {
            builder = builder.header(&h[..], &v[..]);
        }

        let json = builder.await?.body_json().await;
        match json {
            Ok(v) => Ok(v),
            Err(e) => Err(format_err!("Failed to request Tissue: {}", e).into()),
        }
    }
}
