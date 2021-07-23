mod lametric;
mod schema;

use lametric::{LaMetricFrame, LaMetricResponse, LM_ICON_CLOCK, LM_ICON_SPERM, LM_ICON_TISSUE};
use schema::AppParameters;

use anyhow::Result;
use chrono::prelude::*;
use log::info;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::to_string as to_json_string;
use tide::{http::StatusCode, Error as TideError, Request, Response, Result as TideResult};
use tissue_rs::{CheckinBuilder, CheckinResponse, IncomingEndpoint};

const USER_AGENT: &str = concat!("Jerkounter/LaMetric ", env!("CARGO_PKG_VERSION"));
static RE_CHECKINS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#">\s*通算回数\s*:\s*([\d,]+)\s*回\s*</"#).expect("Invalid regex"));
static RE_INTERVAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#">\s*([\d]+)\s*日\s*([\d]+)\s*時間\s*([\d]+)\s*分経過</"#).expect("Invalid regex")
});

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut app = tide::new();
    app.at("/user").get(fetch_user);
    app.at("/checkin").get(send_checkin);
    app.listen("127.0.0.1:8000").await?;

    Ok(())
}

/// Responses user information.
async fn fetch_user(request: Request<()>) -> TideResult {
    let query: AppParameters = request.query()?;

    info!("Fetching user {}", query.name);
    let (checkins, interval) = fetch_user_info(&query.name).await?;
    let response = LaMetricResponse {
        frames: vec![
            LaMetricFrame {
                index: 0,
                icon: Some(LM_ICON_TISSUE),
                text: query.name,
            },
            LaMetricFrame {
                index: 1,
                icon: Some(LM_ICON_SPERM),
                text: format!("{} 回チェックイン", checkins),
            },
            LaMetricFrame {
                index: 2,
                icon: Some(LM_ICON_CLOCK),
                text: format!("{} 日 {} 時間 {} 分", interval.0, interval.1, interval.2),
            },
        ],
    };
    Ok(Response::builder(StatusCode::Ok)
        .body(to_json_string(&response)?)
        .build())
}

/// Does the Action Buttion process.
async fn send_checkin(request: Request<()>) -> TideResult {
    let query: AppParameters = request.query()?;

    info!("Sending checkin to {}", query.name);
    let tissue = IncomingEndpoint::new(&query.webhook_token);
    let mut builder = CheckinBuilder::with_datetime(Local::now());
    builder.note("Sent from LaMetric Time")?;

    let checkin = builder.build();
    match tissue.send_checkin(&checkin).await {
        Ok(CheckinResponse::Success(_)) => Ok("".into()),
        _ => Ok("Error occurred".into()),
    }
}

/// Fetches information from Tissue.
async fn fetch_user_info(username: &str) -> Result<(i32, (i32, i32, i32)), TideError> {
    let url = format!("https://shikorism.net/user/{}", username);
    let user_html = surf::get(url)
        .header("User-Agent", USER_AGENT)
        .await?
        .body_string()
        .await?;

    match (
        RE_CHECKINS.captures(&user_html),
        RE_INTERVAL.captures(&user_html),
    ) {
        (Some(cm), Some(im)) => {
            let c = cm
                .get(1)
                .map_or("0", |m| m.as_str())
                .replace(',', "")
                .parse::<i32>()?;
            let id = im.get(1).map_or("0", |m| m.as_str()).parse::<i32>()?;
            let ih = im.get(2).map_or("0", |m| m.as_str()).parse::<i32>()?;
            let im = im.get(3).map_or("0", |m| m.as_str()).parse::<i32>()?;
            Ok((c, (id, ih, im)))
        }
        _ => Ok((0, (0, 0, 0))),
    }
}
