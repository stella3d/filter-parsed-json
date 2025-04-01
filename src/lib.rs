use std::collections::VecDeque;

use serde::{Deserialize, de::DeserializeOwned};

#[derive(Debug, Deserialize)]
#[serde(transparent)]
/// an intermediate parsing step between a json array and a Vec<T>,
/// which will drop any array values that fail to parse, without
/// failing to parse the entire array (as would happen with a Vec<T>)
pub struct FilterParsedJsonVec<T: DeserializeOwned>(
    pub VecDeque<serde_json::Value>,
    #[serde(skip)] std::marker::PhantomData<T>,
);

impl<T: DeserializeOwned> Iterator for FilterParsedJsonVec<T> {
    // a more elegant version might do Result<T> and bubble the error,
    // but this is designed to simply throw away invalid values and continue
    type Item = Option<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(|v| match serde_json::from_value(v) {
            Ok(val) => Some(val),
            Err(_) => None,
        })
    }
}

impl<T: DeserializeOwned> From<FilterParsedJsonVec<T>> for Vec<T> {
    fn from(val: FilterParsedJsonVec<T>) -> Self {
        val.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unparseable_int_filtering() {
        let json_str = r#"[-2, 2, "3rd", 4, true, 8, 80000]"#;
        let vec: FilterParsedJsonVec<u16> = serde_json::from_str(json_str).unwrap();
        let parsed: Vec<u16> = vec.into();
        // -2, "3rd", true, 80000 are unparseable as u16, so should be dropped
        assert_eq!(parsed, vec![2, 4, 8]);
    }
}
