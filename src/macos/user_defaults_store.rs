// Keys match the earlier Swift app (same bundle identifier), so its saved settings carry over.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
#[cfg(test)]
use objc2::AnyThread;
use objc2_foundation::{NSArray, NSString, NSUserDefaults};

use crate::domain::settings::KeyValueStore;

pub struct UserDefaultsStore {
    defaults: Retained<NSUserDefaults>,
}

impl UserDefaultsStore {
    pub fn standard() -> Self {
        Self {
            defaults: NSUserDefaults::standardUserDefaults(),
        }
    }

    #[cfg(test)]
    pub fn suite(name: &str) -> Self {
        let suite_name = NSString::from_str(name);
        let defaults =
            NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&suite_name))
                .expect("NSUserDefaults(suiteName:) should not return nil for a valid suite name");
        Self { defaults }
    }
}

impl KeyValueStore for UserDefaultsStore {
    fn get_string_array(&self, key: &str) -> Option<Vec<String>> {
        let key = NSString::from_str(key);
        let array = self.defaults.stringArrayForKey(&key)?;
        Some(
            array
                .to_vec()
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        )
    }

    fn get_bool(&self, key: &str) -> bool {
        let key = NSString::from_str(key);
        self.defaults.boolForKey(&key)
    }

    fn set_string_array(&mut self, key: &str, values: &[String]) {
        let key = NSString::from_str(key);
        let ns_strings: Vec<Retained<NSString>> =
            values.iter().map(|s| NSString::from_str(s)).collect();
        let array: Retained<NSArray<NSString>> = NSArray::from_retained_slice(&ns_strings);
        let any_object: &AnyObject = &array;
        // SAFETY: `array` is an `NSArray<NSString>`, a valid property-list
        // object type for `NSUserDefaults`.
        unsafe {
            self.defaults.setObject_forKey(Some(any_object), &key);
        }
    }

    fn set_bool(&mut self, key: &str, value: bool) {
        let key = NSString::from_str(key);
        self.defaults.setBool_forKey(value, &key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SUITE_NAME: &str = "baz.CountdownTimerBar.tests.user_defaults_store";

    fn clear_test_suite() {
        NSUserDefaults::standardUserDefaults()
            .removePersistentDomainForName(&NSString::from_str(TEST_SUITE_NAME));
    }

    #[test]
    fn round_trips_string_array_and_bool_through_a_real_user_defaults_suite() {
        clear_test_suite();

        let mut store = UserDefaultsStore::suite(TEST_SUITE_NAME);
        store.set_string_array("focusTimers", &["120".to_string(), "240".to_string()]);
        store.set_bool("soundOn", true);
        assert_eq!(
            store.get_string_array("focusTimers"),
            Some(vec!["120".to_string(), "240".to_string()])
        );
        assert!(store.get_bool("soundOn"));
        assert!(!store.get_bool("missingKey"));

        clear_test_suite();
    }
}
