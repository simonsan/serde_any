extern crate serde_any;
extern crate serde_value;

#[macro_use]
extern crate serde;

use std::collections::hash_map::HashMap;
use serde_value::Value;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct ItemValue {
    #[serde(rename = "$value")]
    value: Value,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Root {
    #[serde(flatten)]
    extra: HashMap<String, ItemValue>,
}


#[test]
fn xml_root_values() {
    let expected = {
        let mut expected = Root {
            extra: HashMap::new(),
        };
        expected.extra.insert("Foo".to_string(), ItemValue { value: Value::String("42".to_string()) });
        expected
    };

    let output: Root = serde_any::from_str("<Root><Foo>42</Foo></Root>", serde_any::Format::Xml).unwrap();
    assert_eq!(output, expected);
}
