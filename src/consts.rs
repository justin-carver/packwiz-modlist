pub const MODRINTH_API: &str = "https://api.modrinth.com/v2";
/// Organizations are exposed only on v3, which Modrinth documents as unstable
/// and subject to change -- so it is used for that one lookup and nothing else,
/// and a failure there degrades to a warning rather than an error.
pub const MODRINTH_API_V3: &str = "https://api.modrinth.com/v3";
pub const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
pub const USER_AGENT: &str = "packwiz-modlist/user-agent-string@1.0.0";
pub(crate) const CURSEFORGE_API_KEY: &str = env!("CF_API_KEY");
