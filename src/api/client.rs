use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

use super::models::*;

const AFLAPI: &str = "https://aflapi.afl.com.au";
const CFS: &str = "https://api.afl.com.au";
const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) afl-tui/0.1 (terminal fixture viewer)";
const AFL_COMPETITION_ID: u32 = 1;

pub struct AflClient {
    http: reqwest::Client,
    token: Mutex<Option<String>>,
}

impl AflClient {
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(15))
            .build()?;
        Ok(Self {
            http,
            token: Mutex::new(None),
        })
    }

    pub async fn comp_seasons(&self) -> Result<Vec<CompSeason>> {
        let url =
            format!("{AFLAPI}/afl/v2/competitions/{AFL_COMPETITION_ID}/compseasons?pageSize=20");
        let res: CompSeasonsResponse = self.get_json(&url).await?;
        Ok(res.comp_seasons)
    }

    pub async fn round_matches(
        &self,
        comp_season_id: u32,
        round: u32,
    ) -> Result<Vec<FixtureMatch>> {
        let url = format!(
            "{AFLAPI}/afl/v2/matches?competitionId={AFL_COMPETITION_ID}&compSeasonId={comp_season_id}&roundNumber={round}&pageSize=30"
        );
        let res: MatchesResponse = self.get_json(&url).await?;
        Ok(res.matches)
    }

    pub async fn ladder(&self, comp_season_id: u32) -> Result<Vec<LadderEntry>> {
        let url = format!("{AFLAPI}/afl/v2/compseasons/{comp_season_id}/ladders");
        let res: LadderResponse = self.get_json(&url).await?;
        Ok(res
            .ladders
            .into_iter()
            .flat_map(|ladder| ladder.entries)
            .collect())
    }

    pub async fn match_item(&self, provider_id: &str) -> Result<MatchItem> {
        let url = format!("{CFS}/cfs/afl/matchItem/{provider_id}");
        self.cfs_get(&url).await
    }

    pub async fn player_stats(&self, provider_id: &str) -> Result<PlayerStatsResponse> {
        let url = format!("{CFS}/cfs/afl/playerStats/match/{provider_id}");
        self.cfs_get(&url).await
    }

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let res = self.http.get(url).send().await?.error_for_status()?;
        res.json::<T>()
            .await
            .with_context(|| format!("decoding response from {url}"))
    }

    /// GET a cfs endpoint with the X-media-mis-token header, refreshing the
    /// token and retrying once if the cached one has expired.
    async fn cfs_get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let token = self.token(false).await?;
        let res = self
            .http
            .get(url)
            .header("X-media-mis-token", &token)
            .send()
            .await?;
        let res = if matches!(
            res.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            let token = self.token(true).await?;
            self.http
                .get(url)
                .header("X-media-mis-token", &token)
                .send()
                .await?
        } else {
            res
        };
        let res = res.error_for_status()?;
        res.json::<T>()
            .await
            .with_context(|| format!("decoding response from {url}"))
    }

    async fn token(&self, force_refresh: bool) -> Result<String> {
        let mut guard = self.token.lock().await;
        if !force_refresh && let Some(t) = guard.as_ref() {
            return Ok(t.clone());
        }
        let url = format!("{CFS}/cfs/afl/WMCTok");
        let res: TokenResponse = self
            .http
            .post(&url)
            .header("Content-Length", "0")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .context("decoding WMCTok token response")?;
        *guard = Some(res.token.clone());
        Ok(res.token)
    }
}
