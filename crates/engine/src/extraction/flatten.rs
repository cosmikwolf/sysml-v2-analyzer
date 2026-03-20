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

    // Note: Individual flatten_metadata_value tests removed —
    // now covered by property tests in property_tests module below.

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

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use sysml_v2_adapter::MetadataValue;

    /// Strategy for generating arbitrary MetadataValue trees (depth-limited).
    fn metadata_value(depth: u32) -> impl Strategy<Value = MetadataValue> {
        let leaf = prop_oneof![
            any::<bool>().prop_map(MetadataValue::Boolean),
            any::<i64>().prop_map(MetadataValue::Integer),
            "[a-zA-Z_][a-zA-Z0-9_]{0,20}".prop_map(|s| MetadataValue::String(s)),
            ("[A-Z][a-zA-Z]{0,10}", "[a-z][a-zA-Z]{0,10}")
                .prop_map(|(t, v)| MetadataValue::EnumRef {
                    enum_type: t,
                    variant: v,
                }),
        ];

        if depth == 0 {
            leaf.boxed()
        } else {
            prop_oneof![
                leaf.clone(),
                prop::collection::vec(metadata_value(depth - 1), 0..4)
                    .prop_map(MetadataValue::Tuple),
            ]
            .boxed()
        }
    }

    proptest! {
        #[test]
        fn flatten_never_returns_null(val in metadata_value(2)) {
            let result = flatten_metadata_value(&val);
            prop_assert!(!result.is_null(), "flatten produced null for {:?}", val);
        }

        #[test]
        fn flatten_boolean_stays_boolean(b in any::<bool>()) {
            let val = MetadataValue::Boolean(b);
            let result = flatten_metadata_value(&val);
            prop_assert!(result.is_boolean(), "Boolean({}) became {:?}", b, result);
            prop_assert_eq!(result.as_bool().unwrap(), b);
        }

        #[test]
        fn flatten_integer_stays_number(n in any::<i64>()) {
            let val = MetadataValue::Integer(n);
            let result = flatten_metadata_value(&val);
            prop_assert!(result.is_number(), "Integer({}) became {:?}", n, result);
            prop_assert_eq!(result.as_i64().unwrap(), n);
        }

        #[test]
        fn flatten_string_stays_string(s in "[a-zA-Z0-9_ ]{0,30}") {
            let val = MetadataValue::String(s.clone());
            let result = flatten_metadata_value(&val);
            prop_assert!(result.is_string(), "String became {:?}", result);
            prop_assert_eq!(result.as_str().unwrap(), s.as_str());
        }

        #[test]
        fn flatten_enum_ref_contains_double_colon(
            t in "[A-Z][a-zA-Z]{0,10}",
            v in "[a-z][a-zA-Z]{0,10}"
        ) {
            let val = MetadataValue::EnumRef {
                enum_type: t.clone(),
                variant: v.clone(),
            };
            let result = flatten_metadata_value(&val);
            let s = result.as_str().unwrap();
            prop_assert!(
                s.contains("::"),
                "EnumRef('{}', '{}') → '{}' missing '::'",
                t, v, s,
            );
            prop_assert_eq!(s, format!("{}::{}", t, v));
        }

        #[test]
        fn flatten_tuple_produces_array(vals in prop::collection::vec(metadata_value(0), 0..5)) {
            let val = MetadataValue::Tuple(vals.clone());
            let result = flatten_metadata_value(&val);
            prop_assert!(result.is_array(), "Tuple became {:?}", result);
            prop_assert_eq!(result.as_array().unwrap().len(), vals.len());
        }
    }
}
