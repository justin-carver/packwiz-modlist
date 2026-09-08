<!-- [![crates.io](https://img.shields.io/crates/v/packwiz-modlist.svg)](https://crates.io/crates/packwiz-modlist) -->
[![license](https://img.shields.io/github/license/Ricky12Awesome/packwiz-modlist)](https://github.com/Ricky12Awesome/packwiz-modlist/blob/main/LICENSE)

# Packwiz ModList

Project original forked from [Rick12Awesome's:  packwiz-modlist](https://github.com/Ricky12Awesome/packwiz-modlist/). (Thanks Rick!!)

A collection of utility command-line flags/commands that augments the original `packwiz` CLI app. 

## Features

- Creates a **Minecraft** modlist from [packwiz](https://packwiz.infra.link/). 

    The format of the modlist can be customized to your choosing, based on a collection of available placeholder/template strings, and it's output stored in a file or piped into other programs.

    See the [Formatting](#Formatting) section for more information.

## Formatting

`--format` / `-f` takes a string literal with `{PLACEHOLDER}` holes in it, one
per field the cache holds. 

The default modpack output is (Markdown List format):

```
- [{NAME}]({URL}) - {DESCRIPTION}\n
```

```sh
# Markdown table rows
packwizml -f '| {NAME} | {AUTHORS} | {LICENSE_ID} |\n'
# HTML list-item anchor tags with a newline
packwizml -f '<li><a href="{URL}">{NAME}</a> — {DESC}</li>\n'
# Perhaps something a bit more complicated
packwizml -f '| {COUNT}. | <img src="{ICON_URL}" width="128px" /> | <a href="{URL}">{NAME}</a><br/><code>{DESC}</code><br/><br/><i>by {AUTHORS_MD}</i> |\n'
```

![alt text](.github/assets/complex-format.png)

Backslash escapes (`\n`, `\t`, `\r`, `\0`, `\\`, `\{`, `\}`) are resolved by
packwizml rather than by the shell, so quote the template and write `\n`
wherever you want a line break — nothing is appended for you. Placeholder names
are case-insensitive, and a bad template is rejected before any API calls are
made.

| Placeholder | Value |
| --- | --- |
| `{ID}` | Project id (Modrinth base62, CurseForge numeric) |
| `{SLUG}` | URL slug, e.g. `sodium` |
| `{NAME}`, `{TITLE}` | Project title |
| `{DESCRIPTION}`, `{DESC}` | Short description / summary |
| `{URL}` | Project page on Modrinth/CurseForge |
| `{ICON_URL}` | Project icon image |
| `{SOURCE_URL}` | Source repository |
| `{ISSUES_URL}` | Issue tracker |
| `{WIKI_URL}` | Wiki / documentation |
| `{LICENSE}` | License name |
| `{LICENSE_ID}` | License id, e.g. `MIT` |
| `{LICENSE_URL}` | License text |
| `{AUTHORS}` | Author names, or the owning organization, comma separated |
| `{AUTHOR_URLS}` | Author pages, comma separated |
| `{AUTHORS_MD}` | Authors as markdown links |
| `{COUNT}` | This mod's position in the list, starting at 1 |

A placeholder with no value for a given mod renders as an empty string. 

### Formatting Notes

Line breaks *inside* a value are collapsed to single spaces before
substitution. Both Modrinth and CurseForge allow them in a description, and one arriving mid-entry
would otherwise split a list item or table row across lines — so the only line
breaks in the output are the ones custom format's request.

CurseForge exposes no license anywhere in its public API, even though it is shown on the project page. `{LICENSE*}` is
therefore empty for CurseForge mods.

Modrinth author names cost extra lookups, because a project is credited to a
*Team* rather than to a list of users:

- Team members come from a bulk `/v2/teams` call, sorted owner-first so the
  credit line is stable between runs.
- A project owned by an *organization* has an empty team, and the site credits
  the organization — so `{AUTHORS}` gets the organization
  (`Forgified Fabric API :: Sinytra`). This is the one place `packwiz-modlist` touches
  Modrinth's `/v3` API, which is documented as unstable, so a failure there
  logs a warning and leaves those authors empty rather than failing the run... perhaps it'll be stable later.

**Neither API lookup runs when every mod is already cached.**

> If you are getting rate-limited by an API, it is advisible to update the cache once all mod changes are finished.

`packwizml --help` prints the same table, generated from the same source.

## Todo

Just a small list of things I'd like to implement that would probably elevate this app just a little bit more:

1. Cache, list, query dependencies for each mod.
2. Ability to have inline notifications regarding when a new update for a specific Minecraft version is available.
3. Display info regarding: (+ # of New Mods) or (- # Deleted mods), and their names, since last cache state.
4. Extend the cache db to include information about when mod was added (may help troubleshoot terrible mod issues!)
5. Bulk Edit / Bulk Modify mods based on regular expressions
6. (Idk if this can be done???) Ability to hook into log files and determine what mods/deps caused previous crashes.