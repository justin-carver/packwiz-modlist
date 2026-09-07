use crate::consts::{CURSEFORGE_API, CURSEFORGE_API_KEY};
use crate::error::Error;
use crate::request::{post, CurseForgeId};
use minreq::Request;
use serde::{Deserialize, Serialize};

pub type Mods = Vec<Mod>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
  pub id: u32,
  pub name: String,
  pub slug: String,
  pub links: ModLinks,
  pub summary: String,
  pub authors: Vec<ModAuthor>,
  pub logo: ModLogo,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLinks {
  #[serde_as(as = "serde_with::NoneAsEmptyString")]
  pub website_url: Option<String>,
  #[serde_as(as = "serde_with::NoneAsEmptyString")]
  pub wiki_url: Option<String>,
  #[serde_as(as = "serde_with::NoneAsEmptyString")]
  pub issues_url: Option<String>,
  #[serde_as(as = "serde_with::NoneAsEmptyString")]
  pub source_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModAuthor {
  pub id: u32,
  pub name: String,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLogo {
  pub id: u32,
  pub mod_id: u32,
  pub title: String,
  pub description: String,
  pub thumbnail_url: String,
}

pub fn post_curseforge(endpoint: &str) -> Request {
  post(format!("{CURSEFORGE_API}{endpoint}")).with_header("x-api-key", CURSEFORGE_API_KEY)
}

pub fn get_curseforge_mods(ids: Vec<CurseForgeId>) -> Result<Mods, Error> {
  #[derive(Serialize, Deserialize, Debug, Clone)]
  #[serde(rename_all = "camelCase")]
  struct ResponseJson {
    data: Vec<Mod>,
  }

  let body = serde_json::json!({ "modIds": ids, "filterPcOnly": true });
  let response = post_curseforge("/mods").with_json(&body)?.send()?;

  match response.status_code {
    200 => response
      .json::<ResponseJson>()
      .map_err(|err| {
        (match response.as_str() {
          Ok(json) => (json, err).into(),
          Err(err) => err.into(),
        })
      })
      .map(|m| m.data),
    _ => {
      log::debug!(
        "request body sent to CurseForge:\n{}",
        serde_json::to_string_pretty(&body).unwrap_or_else(|_| body.to_string())
      );
      // A 403 from CurseForge carries a zero-byte body, so the headers are the
      // only place left with any signal about which hop rejected the request.
      log::debug!(
        "CurseForge returned {} {} with headers:\n{}",
        response.status_code,
        response.reason_phrase,
        crate::request::describe_headers(&response)
      );
      // The body rides along on the error itself, so it is visible without -vv.
      Err(response.into())
    }
  }
}
