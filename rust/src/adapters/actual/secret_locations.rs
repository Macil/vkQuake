use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

use super::raw_bindings::vec3_t;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Location([i64; 3]);

impl From<&vec3_t> for Location {
    fn from(v: &vec3_t) -> Self {
        Self([v[0] as i64, v[1] as i64, v[2] as i64])
    }
}

static SECRET_LOCATIONS: Lazy<Mutex<HashMap<Location, u16>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
pub unsafe extern "C" fn Secret_ClearLocations() {
    SECRET_LOCATIONS.lock().unwrap().clear();
}

#[no_mangle]
pub unsafe extern "C" fn Secret_RecordLocation(secret: u16, mins: &vec3_t) {
    SECRET_LOCATIONS.lock().unwrap().insert(mins.into(), secret);
}

/// Returns -1 if the secret is not found.
#[no_mangle]
pub unsafe extern "C" fn Secret_GetIndexForLocation(mins: &vec3_t) -> i32 {
    SECRET_LOCATIONS
        .lock()
        .unwrap()
        .get(&mins.into())
        .map(|s| *s as i32)
        .unwrap_or(-1)
}
