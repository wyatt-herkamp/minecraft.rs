pub mod string_or_array {
    use serde::{Deserializer, Serializer};
    #[inline(always)]
    pub fn serialize<S>(value: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.is_empty() {
            serializer.serialize_none()
        } else if value.len() == 1 {
            serializer.serialize_str(&value[0])
        } else {
            serializer.collect_seq(value.iter())
        }
    }
    struct StringOrArrayVisitor;
    impl<'de> serde::de::Visitor<'de> for StringOrArrayVisitor {
        type Value = Vec<String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or array of strings")
        }
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_string()])
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![])
        }
        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            serde::de::Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
        }
    }
    #[inline(always)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StringOrArrayVisitor)
    }
}
