use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Default, Clone)]
pub struct VectorData {
    pub vector: Vec<f32>,
    pub embedding_type: String,
}

impl VectorData {
    fn new(vector: Vec<f32>, embedding_type: String) -> VectorData {
        VectorData {
            vector: vector,
            embedding_type: embedding_type,
        }
    }
}

impl Serialize for VectorData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("VectorData", 2)?;
        state.serialize_field("vector", &self.vector)?;
        state.serialize_field("embedding_type", &self.embedding_type)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for VectorData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Vector,
            EmbeddingType,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`vector` or `embedding_type`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "vector" => Ok(Field::Vector),
                            "embedding_type" => Ok(Field::EmbeddingType),
                            _ => Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct VectorDataVisitor;

        impl<'de> Visitor<'de> for VectorDataVisitor {
            type Value = VectorData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct VectorData")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let vector = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let embedding_type = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(VectorData::new(vector, embedding_type))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut vector = None;
                let mut embedding_type = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Vector => {
                            if vector.is_some() {
                                return Err(de::Error::duplicate_field("vector"));
                            }
                            vector = Some(map.next_value()?);
                        }
                        Field::EmbeddingType => {
                            if embedding_type.is_some() {
                                return Err(de::Error::duplicate_field("embedding_type"));
                            }
                            embedding_type = Some(map.next_value()?);
                        }
                    }
                }
                let vector = vector.ok_or_else(|| de::Error::missing_field("vector"))?;
                let embedding_type =
                    embedding_type.ok_or_else(|| de::Error::missing_field("embedding_types"))?;
                Ok(VectorData::new(vector, embedding_type))
            }
        }

        const FIELDS: &'static [&'static str] = &["vector", "embedding_type"];
        deserializer.deserialize_struct("VectorData", FIELDS, VectorDataVisitor)
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug, Serialize, Deserialize)]
pub enum DataType {
    #[default]
    Text,
    Image,
    Audio,
    Blob,
}

#[derive(Default, Debug, Clone)]
pub struct Data {
    pub vector: VectorData,
    pub payload: String,
    pub data_type: DataType,
}

impl Data {
    fn new(vector: VectorData, payload: String, data_type: DataType) -> Data {
        Data {
            vector: vector,
            payload: payload,
            data_type: data_type,
        }
    }
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Data", 3)?;
        state.serialize_field("vector", &self.vector)?;
        state.serialize_field("payload", &self.payload)?;
        state.serialize_field("data_type", &self.data_type)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Vector,
            Payload,
            DataType,
        }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`vector` or `payload` or `data_type`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "vector" => Ok(Field::Vector),
                            "payload" => Ok(Field::Payload),
                            "data_type" => Ok(Field::DataType),
                            _ => Err(de::Error::unknown_field(v, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct DataVisitor;

        impl<'de> Visitor<'de> for DataVisitor {
            type Value = Data;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Data")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let vector = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let payload = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let data_type = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(Data::new(vector, payload, data_type))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut vector = None;
                let mut payload = None;
                let mut data_type = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Vector => {
                            if vector.is_some() {
                                return Err(de::Error::duplicate_field("vector"));
                            }
                            vector = Some(map.next_value()?);
                        }
                        Field::Payload => {
                            if payload.is_some() {
                                return Err(de::Error::duplicate_field("payload"));
                            }
                            payload = Some(map.next_value()?);
                        }
                        Field::DataType => {
                            if data_type.is_some() {
                                return Err(de::Error::duplicate_field("data_type"));
                            }
                            data_type = Some(map.next_value()?);
                        }
                    }
                }
                let vector = vector.ok_or_else(|| de::Error::missing_field("vector"))?;
                let payload = payload.ok_or_else(|| de::Error::missing_field("payload"))?;
                let data_type = data_type.ok_or_else(|| de::Error::missing_field("data_type"))?;
                Ok(Data::new(vector, payload, data_type))
            }
        }

        const FIELDS: &'static [&'static str] = &["vector", "payload", "data_type"];
        deserializer.deserialize_struct("VectorData", FIELDS, DataVisitor)
    }
}
