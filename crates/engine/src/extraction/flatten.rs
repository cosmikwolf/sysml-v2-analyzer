//! Metadata value flattening: adapter `MetadataValue` → `serde_json::Value`.

use std::collections::HashMap;

use serde_json::json;
use sysml_v2_adapter::{MetadataAnnotation, MetadataValue};

/// Convert a single `MetadataValue` to a JSON value.
pub(crate) fn flatten_metadata_value(val: &MetadataValue) -> serde_json::Value {
    match val {
        MetadataValue::EnumRef {
            enum_type,
            variant,
        } => json!(format!("{enum_type}::{variant}")),
        MetadataValue::Boolean(b) => json!(b),
        MetadataValue::Integer(n) => json!(n),
        MetadataValue::String(s) => json!(s),
        MetadataValue::Tuple(vals) => {
            json!(vals.iter().map(flatten_metadata_value).collect::<Vec<_>>())
        }
    }
}

/// Flatten a list of metadata annotations into a nested map:
/// `annotation_name → { field_name → json_value }`.
pub(crate) fn flatten_annotations(
    annotations: &[MetadataAnnotation],
) -> HashMap<String, HashMap<String, serde_json::Value>> {
    let mut result = HashMap::new();
    for ann in annotations {
        let mut fields = HashMap::new();
        for field in &ann.fields {
            fields.insert(field.name.clone(), flatten_metadata_value(&field.value));
        }
        result.insert(ann.name.clone(), fields);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysml_v2_adapter::{MetadataField, MetadataValue};

    #[test]
    fn test_flatten_enum_ref() {
        let val = MetadataValue::EnumRef {
            enum_type: "AllocationKind".to_string(),
            variant: "static_alloc".to_string(),
        };
        assert_eq!(
            flatten_metadata_value(&val),
            json!("AllocationKind::static_alloc")
        );
    }

    #[test]
    fn test_flatten_boolean() {
        assert_eq!(flatten_metadata_value(&MetadataValue::Boolean(true)), json!(true));
        assert_eq!(flatten_metadata_value(&MetadataValue::Boolean(false)), json!(false));
    }

    #[test]
    fn test_flatten_integer() {
        assert_eq!(flatten_metadata_value(&MetadataValue::Integer(42)), json!(42));
    }

    #[test]
    fn test_flatten_string() {
        assert_eq!(
            flatten_metadata_value(&MetadataValue::String("hello".to_string())),
            json!("hello")
        );
    }

    #[test]
    fn test_flatten_tuple() {
        let val = MetadataValue::Tuple(vec![
            MetadataValue::String("a".to_string()),
            MetadataValue::Integer(1),
        ]);
        assert_eq!(flatten_metadata_value(&val), json!(["a", 1]));
    }

    #[test]
    fn test_flatten_annotations() {
        let annotations = vec![MetadataAnnotation {
            name: "MemoryModel".to_string(),
            fields: vec![
                MetadataField {
                    name: "allocation".to_string(),
                    value: MetadataValue::EnumRef {
                        enum_type: "AllocationKind".to_string(),
                        variant: "static_alloc".to_string(),
                    },
                },
                MetadataField {
                    name: "maxInstances".to_string(),
                    value: MetadataValue::Integer(1),
                },
            ],
        }];

        let result = flatten_annotations(&annotations);
        let mm = result.get("MemoryModel").expect("should have MemoryModel");
        assert_eq!(mm.get("allocation").unwrap(), &json!("AllocationKind::static_alloc"));
        assert_eq!(mm.get("maxInstances").unwrap(), &json!(1));
    }
}
