mod lametric;
mod schema;
mod tissue;

use lametric::{LaMetricFrame, LaMetricResponse, LM_ICON_CLOCK, LM_ICON_SPERM, LM_ICON_TISSUE};
use schema::AppParameters;
use tissue::{RE_CHECKINS, RE_INTERVAL, SurfRequester};

use anyhow::Result;
use chrono::{prelude::*, Duration};
use log::info;
use serde_json::to_string as to_json_string;
use std::env;
use tide::{http::StatusCode, Error as TideError, Request, Response, Result as TideResult};
use tissue_rs::{CheckinBuilder, CheckinResponse, IncomingEndpoint};

const USER_AGENT: &str = concat!("Jerkounter/LaMetric ", env!("CARGO_PKG_VERSION"));

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    info!("User-Agent: {}", USER_AGENT);
    let listen_at = env::var("LISTEN_AT")?;

    let mut app = tide::new();
    app.at("/user").get(fetch_user);
    app.at("/checkin").get(send_checkin);
    app.listen(listen_at).await?;

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
                text: format!(
                    "{} 日 {} 時間 {} 分",
                    interval.num_days(),
                    interval.num_hours() % 24,
                    interval.num_minutes() % 60,
                ),
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
    let mut tissue = IncomingEndpoint::new(&query.webhook_token, SurfRequester);
    let mut builder = CheckinBuilder::with_datetime(Local::now());
    builder.note("Sent from LaMetric Time")?;

    let checkin = builder.build();
    match tissue.send_checkin(&checkin).await {
        Ok(CheckinResponse::Success(_)) => Ok("".into()),
        _ => Ok("Error occurred".into()),
    }
}

/// Fetches information from Tissue.
async fn fetch_user_info(username: &str) -> Result<(usize, Duration), TideError> {
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
        (Some(cmatch), Some(imatch)) => {
            let checkins = cmatch
                .get(1)
                .map_or("0", |m| m.as_str())
                .replace(',', "")
                .parse()?;
            let interval = Duration::days(imatch.get(1).map_or("0", |m| m.as_str()).parse()?)
                + Duration::hours(imatch.get(2).map_or("0", |m| m.as_str()).parse()?)
                + Duration::minutes(imatch.get(3).map_or("0", |m| m.as_str()).parse()?);

            Ok((checkins, interval))
        }
        _ => Ok((0, Duration::zero())),
    }
}
