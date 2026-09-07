use crate::consts::USER_AGENT;
use crate::parser::{ParsedCurseForgeId, ParsedModrinthId};
use minreq::{Request, URL};
use serde::{Deserialize, Serialize};

pub mod curseforge;
pub mod modrinth;

pub fn get<T: Into<URL>>(url: T) -> Request {
  minreq::get(url)
    .with_header("User-Agent", USER_AGENT)
    .with_header("Content-Type", "application/json")
    .with_header("Accept", "application/json")
}

pub fn post<T: Into<URL>>(url: T) -> Request {
  minreq::post(url)
    .with_header("User-Agent", USER_AGENT)
    .with_header("Content-Type", "application/json")
    .with_header("Accept", "application/json")
}

/// Renders a true response body for diagnostics
pub fn describe_body(response: &minreq::Response) -> String {
  match response.as_str() {
    Ok(text) => render_body(text),
    Err(_) => format!("<{} bytes of non-UTF-8 body>", response.as_bytes().len()),
  }
}

/// The pure half of [`describe_body`], split out so it can be tested without
/// constructing a live [`minreq::Response`], byte-for-byte.
fn render_body(text: &str) -> String {
  if text.trim().is_empty() {
    return "<empty body -- the server sent no content>".to_string();
  }

  text.to_string()
}

#[cfg(test)]
mod tests {
  use super::render_body;

  #[test]
  fn empty_body_is_called_out_explicitly() {
    assert_eq!(
      render_body(""),
      "<empty body -- the server sent no content>"
    );
    assert_eq!(
      render_body("   \n "),
      "<empty body -- the server sent no content>"
    );
  }

  #[test]
  fn json_body_is_returned_verbatim_with_key_order_intact() {
    let body = r#"{"error":"request_error","details":["a","b"]}"#;

    // Key order must survive: serde_json would sort these alphabetically.
    assert_eq!(render_body(body), body);
  }

  #[test]
  fn non_json_body_is_passed_through_verbatim() {
    let html = "<html><body>502 Bad Gateway</body></html>";

    assert_eq!(render_body(html), html);
  }
}

/// Renders response headers, sorted for stable output.
///
/// When a body is empty the headers carry what signal there is, like: rate-limits
/// counters, CDN markers, and which hop actually rejected the request.
pub fn describe_headers(response: &minreq::Response) -> String {
  let mut headers: Vec<_> = response.headers.iter().collect();
  headers.sort_by(|a, b| a.0.cmp(b.0));

  headers
    .into_iter()
    .map(|(name, value)| format!("  {name}: {value}"))
    .collect::<Vec<_>>()
    .join("\n")
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModrinthId(pub String);

impl From<&str> for ModrinthId {
  fn from(id: &str) -> Self {
    Self(id.to_string())
  }
}

impl From<String> for ModrinthId {
  fn from(id: String) -> Self {
    Self(id)
  }
}

impl From<ParsedModrinthId> for ModrinthId {
  fn from(id: ParsedModrinthId) -> Self {
    Self(id.id)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurseForgeId(pub i32);

impl From<i32> for CurseForgeId {
  fn from(id: i32) -> Self {
    Self(id)
  }
}

impl From<ParsedCurseForgeId> for CurseForgeId {
  fn from(id: ParsedCurseForgeId) -> Self {
    Self(id.id)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
  pub id: String,
  pub slug: String,
  pub title: String,
  pub description: String,
  /// [Option] because Curseforge doesn't offer a simple way to get a
  /// license for a project even though it's right on the project page
  pub license: Option<License>,
  /// Will be empty since modrinth handles authors in a different way
  /// by using teams, and currently there is no way to bulk get teams
  /// https://github.com/modrinth/labrinth/issues/331
  // TODO: Revisit this issue, perhaps
  pub authors: Vec<Author>,
  pub icon_url: Option<String>,
  pub source_url: Option<String>,
  pub issues_url: Option<String>,
  pub wiki_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Author {
  pub name: String,
  pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct License {
  pub id: String,
  pub name: String,
  pub url: Option<String>,
}

impl From<modrinth::Project> for Mod {
  fn from(project: modrinth::Project) -> Self {
    Self {
      id: project.id,
      slug: project.slug,
      title: project.title,
      description: project.description,
      license: Some(License {
        id: project.license.id,
        name: project.license.name,
        url: project.license.url,
      }),
      authors: Vec::new(),
      icon_url: project.icon_url,
      source_url: project.source_url,
      issues_url: project.issue_url,
      wiki_url: project.wiki_url,
    }
  }
}

impl From<curseforge::Mod> for Mod {
  fn from(project: curseforge::Mod) -> Self {
    Self {
      id: project.id.to_string(),
      slug: project.slug,
      title: project.name,
      description: project.summary,
      license: None,
      authors: project
        .authors
        .into_iter()
        .map(|author| Author {
          name: author.name,
          url: author.url,
        })
        .collect(),
      icon_url: project.logo.thumbnail_url.into(),
      source_url: project.links.source_url,
      issues_url: project.links.issues_url,
      wiki_url: project.links.wiki_url,
    }
  }
}
