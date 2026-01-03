#![expect(
    clippy::literal_string_with_formatting_args,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

use std::fmt::Write;

use maud::{DOCTYPE, Markup, PreEscaped, html};
use time::OffsetDateTime;

use crate::twitch::Stream;

fn base(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="robots" content="noindex";
                title { "Findstream" }

                link rel="icon" href="/favicon.svg";

                link
                    rel="stylesheet"
                    href="https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css"
                    integrity="sha256-Z/om3xyp6V2PKtx8BPobFfo9JCV0cOvBDMaLmquRS+4="
                    crossorigin="anonymous";
            }
        }
        body { (content) }
    }
}

fn header() -> Markup {
    html! {
        div.columns {
            div.column.has-text-centered {
                h1.title {
                    a href="/" { "📼 Findstream" }
                }
                p.subtitle { "A better search for Twitch streams" }
            }
        }
    }
}

fn credits() -> Markup {
    html! {
        div.content.has-text-centered {
            p {
                "Made with ❤️ by "
                a href="https://home.dnaka91.rocks" target="_blank" {
                    strong { "Dominik Nakamura" }
                }
                " | Source Code on "
                a href="https://github.com/dnaka91/findstream" target="_blank" {
                    strong { "GitHub" }
                }
                " | Licensed under "
                a href="https://github.com/dnaka91/findstream/blob/main/LICENSE" target="_blank" {
                    strong { "AGPL-3.0-only" }
                }
            }
        }
    }
}

pub fn index() -> Markup {
    base(html! {
        section.hero.is-fullheight {
            div.hero-body {
                div.container {
                    (header())

                    div.columns {
                        div.column.is-6.is-offset-3 {
                            div.notification {
                                "We now have an API that you can use with any HTTP client. For further details, check out the "
                                a href="/api-info" { "info page" }
                                "."
                            }
                        }
                    }

                    div.columns {
                        div.column.is-6.is-offset-3 {
                            form action="/search" method="GET" {
                                div.field.is-horizontal {
                                    div.field-body {
                                        div.field {
                                            div.control.has-icons-left.is-expanded {
                                                div.select.is-fullwidth {
                                                    select name="category" {
                                                        option value="Art" { "Art" }
                                                        option value="BeautyAndBodyArt" {
                                                            "Beauty & Body Art"
                                                        }
                                                        option value="FoodAndDrink" {
                                                            "Food & Drink"
                                                        }
                                                        option value="JustChatting" {
                                                            "Just Chatting"
                                                        }
                                                        option value="MakersAndCrafting" {
                                                            "Makers & Crafting"
                                                        }
                                                        option value="Music" { "Music" }
                                                        option value="Retro" { "Retro" }
                                                        option value="ScienceAndTechnology" {
                                                            "Science & Technology"
                                                        }
                                                        option
                                                            value="SoftwareAndGameDevelopment"
                                                            selected
                                                        { "Software and Game Development" }
                                                        option value="TalkShowsAndPodcasts" {
                                                            "Talk Shows & Podcasts"
                                                        }
                                                    }
                                                }
                                                div.icon.is-small.is-left { "🏷️" }
                                            }
                                        }
                                        div.field {
                                            div.control.has-icons-left.is-expanded {
                                                div.select.is-fullwidth {
                                                    select name="language" {
                                                        option value="" selected { "All" }
                                                        option value="en" { "English" }
                                                        option value="de" { "German" }
                                                        option value="it" { "Italian" }
                                                        option value="jp" { "Japanese" }
                                                        option value="ko" { "Korean" }
                                                        option value="ru" { "Russian" }
                                                        option value="es" { "Spanish" }
                                                    }
                                                }
                                                div.icon.is-small.is-left { "🎤" }
                                            }
                                        }
                                    }
                                }
                                div.field.is-horizontal {
                                    div.field-body {
                                        div.field.has-addons {
                                            div.control.has-icons-left.is-expanded {
                                                input
                                                    .input
                                                    type="text"
                                                    name="query"
                                                    placeholder="Find a stream"
                                                    autofocus;
                                                span.icon.is-small.is-left { "🔍" }
                                            }
                                            div.control {
                                                input.button.is-info type="submit" value="Search";
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div.columns.mt-6 {
                        div.column { (credits()) }
                    }
                }
            }
        }
    })
}

pub fn api_info() -> Markup {
    base(html! {
        section.hero.is-fullheight {
            div.hero-body {
                div.container {
                    (header())

                    div.columns {
                        div.column.is-6.is-offset-3 {
                            div.content {
                                h2 { "Findstream API" }
                                p {
                                    "The following describes the API provided by "
                                    b { "Findstream" }
                                    ", which allows to get the same results available on the \
                                    webpage of this service."
                                }
                                p {
                                    "If possible, please enable "
                                    code { "gzip" }
                                    " compression in your HTTP client to reduce the amount of data \
                                    transferred."
                                }
                                h3 { "Fair use" }
                                p {
                                    "This API is provided free of service. Please "
                                    b { "just be nice" }
                                    " and don't over-use it. If you plan to send a high volume of \
                                    requests, please open an issue at the "
                                    a href="https://github.com/dnaka91/findstream/issues/new" {
                                        "GitHub project"
                                    }
                                    " first."
                                }
                                p {
                                    "In case a large amount of unusual traffic is seen on this \
                                    endpoint, eventually causing the service to hit the rate limit \
                                    of Twitch's API too often, the API may be changed to be \
                                    invite-only without further notice."
                                }
                                h3 { "Request" }
                                p {
                                    "The request must be sent to "
                                    code { "/api/search" }
                                    " with the "
                                    code { "POST" }
                                    " method and a JSON body. "
                                    "Fields "
                                    code { "category" }
                                    " and "
                                    code { "query" }
                                    " are mandatory, the "
                                    code { "language" }
                                    " field can be used to further restrict results to a specific \
                                    language."
                                }
                                table.table {
                                    thead {
                                        tr {
                                            th { "Field" }
                                            th { "Type" }
                                            th { "Description" }
                                        }
                                    }
                                    tbody {
                                        tr {
                                            td {
                                                code { "category" }
                                            }
                                            td { "String" }
                                            td {
                                                "Twitch category to search in. Possible values are (case-sensitive):"
                                                ul {
                                                    li { "Art" }
                                                    li { "BeautyAndBodyArt" }
                                                    li { "FoodAndDrink" }
                                                    li { "JustChatting" }
                                                    li { "MakersAndCrafting" }
                                                    li { "Music" }
                                                    li { "Retro" }
                                                    li { "ScienceAndTechnology" }
                                                    li { "SoftwareAndGameDevelopment" }
                                                    li { "TalkShowsAndPodcasts" }
                                                }
                                            }
                                        }
                                        tr {
                                            td {
                                                code { "query" }
                                            }
                                            td { "String" }
                                            td {
                                                "The search query. The text will be split by whitespace and stream titles are search to match any of the words."
                                            }
                                        }
                                        tr {
                                            td {
                                                code { "language" }
                                            }
                                            td { "String (optional)" }
                                            td {
                                                "Langauge code ("
                                                a   href="https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes"
                                                { "ISO-639-1" }
                                                ") that results should be limited to."
                                            }
                                        }
                                    }
                                }
                                h4 { "Example" }
                                p {
                                    pre {
                                        (PreEscaped(include_str!("../templates/sample_request.json")))
                                    }
                                }
                                h3 { "Response" }
                                p {
                                    "A successful response is a JSON array with each element having the following fields. Except for "
                                    code { "stream_time" }
                                    ", all fields are always present."
                                }
                                table.table {
                                    thead {
                                        tr {
                                            th { "Field" }
                                            th { "Type" }
                                            th { "Description" }
                                        }
                                    }
                                    tbody {
                                        tr {
                                            td {
                                                code { "title" }
                                            }
                                            td { "String" }
                                            td { "Full stream title as found on Twitch." }
                                        }
                                        tr {
                                            td {
                                                code { "username" }
                                            }
                                            td { "String" }
                                            td {
                                                "Twitch username. Can be combined with the Twitch URL to generate a direct link to the stream. Example: "
                                                code { "https://www.twitch.tv/{username}" }
                                                "."
                                            }
                                        }
                                        tr {
                                            td {
                                                code { "language" }
                                            }
                                            td { "String" }
                                            td {
                                                "Written out full language name (not an ISO code anymore)."
                                            }
                                        }
                                        tr {
                                            td {
                                                code { "stream_time" }
                                            }
                                            td { "i64 (optional)" }
                                            td {
                                                "Duration (in seconds) since the streamer went online."
                                            }
                                        }
                                        tr {
                                            td {
                                                code { "viewer_count" }
                                            }
                                            td { "u64" }
                                            td { "Current amount of viewers." }
                                        }
                                    }
                                }
                                h4 { "Example" }
                                p {
                                    pre {
                                        ({
                                            PreEscaped(
                                                include_str!("../templates/sample_response.json"),
                                            )
                                        })
                                    }
                                }
                            }
                        }
                    }

                    div.columns.mt-6 {
                        div.column { (credits()) }
                    }
                }
            }
        }
    })
}

pub fn results(query_words: &[String], streams: &[Stream], error: bool) -> Markup {
    base(html! {
        section.section {
            div.container {
                (header())

                @if error {
                    div.columns {
                        div.column.has-text-centered {
                            div.box.has-text-white.has-background-danger {
                                span.is-size-4 {
                                    "🤯 Sorry something went wrong, please wait a moment and try again 🤯"
                                }
                            }
                        }
                    }
                } @else if streams.is_empty() {
                    div.columns {
                        div.column.has-text-centered {
                            div.box {
                                span.is-size-4 {
                                    "😭 Nobody streaming "
                                    @for (index, word) in query_words.iter().enumerate() {
                                        @if query_words.len() > 1 && index > 0 {
                                            @if index == query_words.len() - 1 {
                                                (PreEscaped("&nbsp;"))
                                                "or"
                                                (PreEscaped("&nbsp;"))
                                            } @else { "," (PreEscaped("&nbsp;")) }
                                        }
                                        strong { (word) }
                                    }
                                    " right now 😭"
                                }
                            }
                        }
                    }
                } @else {
                    div.fixed-grid.has-3-cols {
                        div.grid {
                            @for stream in streams {
                                div.cell {
                                    div.card {
                                        div.card-image {
                                            figure.image.is-16by9 {
                                                img src=(sized(&stream.thumbnail_url, 640, 360));
                                            }
                                        }
                                        div.card-content {
                                            p.title.is-4 {
                                                a href={ "https://twitch.tv" (stream.user_name) } {
                                                    (stream.title)
                                                }
                                            }
                                            p.subtitle.is-6 { "@" (stream.user_name) }
                                            div.content {
                                                div {
                                                    span.icon { "🎤" }
                                                    span {
                                                        "Speaking "
                                                        strong {
                                                            (crate::lang::translate_iso_639_1(&stream.language))
                                                        }
                                                    }
                                                }
                                                div {
                                                    span.icon { "⏱️" }
                                                    span {
                                                        "Streaming for "
                                                        strong { (since_now(stream.started_at)) }
                                                    }
                                                }
                                                div {
                                                    span.icon { "👀" }
                                                    span {
                                                        "With "
                                                        strong { (stream.viewer_count) " viewers" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div.columns.mt-6 {
                    div.column { (credits()) }
                }
            }
        }
    })
}

fn sized(value: &str, width: u32, height: u32) -> String {
    value
        .replace("{width}", &width.to_string())
        .replace("{height}", &height.to_string())
}

fn since_now(value: Option<OffsetDateTime>) -> String {
    if let Some(value) = value {
        let duration = OffsetDateTime::now_utc() - value;
        let mut buf = String::new();

        match duration.whole_days() {
            0 => {}
            1 => buf.push_str("1 day"),
            d => write!(buf, "{d} days").unwrap(),
        }

        match duration.whole_hours() % 24 {
            0 => {}
            1 => buf.push_str(" 1 hour"),
            h => write!(buf, " {h} hours").unwrap(),
        }

        match duration.whole_minutes() % 60 {
            0 => {}
            1 => buf.push_str(" 1 minute"),
            m => write!(buf, " {m} minutes").unwrap(),
        }

        buf
    } else {
        String::new()
    }
}
