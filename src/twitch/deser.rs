use std::fmt;

use serde::de;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

pub fn opt_datetime<'de, D>(d: D) -> Result<Option<OffsetDateTime>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Option<OffsetDateTime>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("UTC date time")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(
                    OffsetDateTime::parse(value, &Rfc3339).map_err(E::custom)?,
                ))
            }
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    d.deserialize_str(Visitor)
}

pub fn duration<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("duration in seconds")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Duration, E>
        where
            E: de::Error,
        {
            Ok(Duration::seconds(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Duration, E>
        where
            E: de::Error,
        {
            #[allow(clippy::cast_possible_wrap)]
            Ok(Duration::seconds(value as i64))
        }
    }

    d.deserialize_i64(Visitor)
}
