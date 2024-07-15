use std::env::consts::OS;
use serde_json::Value;

pub mod mc_assets;
pub mod mc_libs;
pub mod mc_versions;

pub fn check_rules(element: Value) -> bool{
    let rules = serde_json::from_value::<Option<Vec<Value>>>(element).unwrap();
    if rules.is_none() { return true };

    for rule in rules.unwrap() {
        let action = rule.get("action");
        let os = rule.get("os");
        if action.is_some() && os.is_some() && action.unwrap() == "allow" && os.unwrap() == OS {
            return true
        }
    }
    
    false
    
}