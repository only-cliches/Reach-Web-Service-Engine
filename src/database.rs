use std::char::from_u32;
use std::str::from_utf8;
use std::io::Bytes;
use std::sync::Arc;
use std::error::Error;
use std::fmt::Debug;

use rocksdb::{DB, Options};
use warp::{ Filter, Reply, Rejection, http::StatusCode };
use unicode_normalization::UnicodeNormalization;

use self::DBError::*;

const RECORD_SEP: char = '\u{1E}';
const UNIT_SEP: char = '\u{1F}';
const SEPARATORS: [char; 2] = [ RECORD_SEP, UNIT_SEP ];

// Unicode constants
const LAST_LOW_VALID: u32 = 0xD800 - 1;
const LAST_SURROGATE: u32 = 0xDFFF;
const UNICODE_LIMIT: u32 = 0x10FFFF;

pub fn unicode_successor(letter: char) -> Option<char> {

    let old_hex = letter as u32;
    if old_hex < LAST_LOW_VALID {
        from_u32(old_hex + 1)
    } else if old_hex == LAST_LOW_VALID {
        from_u32(LAST_SURROGATE+1)
    } else if old_hex < LAST_SURROGATE {
        None
    } else if old_hex < UNICODE_LIMIT {
        from_u32(old_hex + 1)
    } else {
        None
    }
}

pub fn string_successor(string: String) -> Option<String> {
    let mut output: String = string.clone();
    output.pop()
        .map(|last| unicode_successor(last))
        .flatten()
        .map(|next| {
            output.push(next);
            output
        })
}

trait LogOk<T> {
    fn log_to_ok(self) -> Option<T>;
}

impl<T, E> LogOk<T> for Result<T, E> where E: Debug {
    fn log_to_ok(self) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => { eprintln!("{:?}", e); None }
        }
    }
}

pub fn kv_filter(domain: String) -> impl warp::Filter<Extract = (impl Reply,), Error = Rejection> + Clone {

    let db_arc = Arc::new(DB::open_default(domain).unwrap());
    let db_middleware = warp::any().map(move || Arc::clone(&db_arc));


    let get_table = warp::path("table")
        .and(warp::path::param())
        .and(db_middleware.clone())
        .map(|table: String, db: Arc<DB>| {

            DBPath::new(table, None)
                .map_err(|e| {
                    eprintln!("{:?}", e);
                    warp::reject::reject()
                }).ok().map(|path| {
                    db.get(path.to_string().as_bytes())
                        .map_err(|e| {
                            eprintln!("{:?}", e);
                        }).ok()
                }).flatten().map(|value| {
                    value.map(|bytes| {
                            from_utf8(bytes.as_ref())
                            .map(|s| warp::reply::with_status(s.to_string(), StatusCode::OK))
                            .map_err(|e| {
                                eprintln!("{:?}", e);
                            }).ok()
                        }).flatten()
                }).flatten()
                .unwrap_or_else(|| warp::reply::with_status("Not Found".to_string(), StatusCode::NOT_FOUND))
        });

        get_table
}

#[derive(Debug, PartialEq, Eq)]
enum DBError {
    ZeroLengthTable,
    AsciiSeparatorInTableName,
    AsciiSeparatorInKeyName,
    ZeroLengthKey,
}

impl ToString for DBError {
    fn to_string(&self) -> String {
        match self {
            ZeroLengthTable => "ZERO LENGTH TABLE",
            AsciiSeparatorInTableName => "ASCII SEPARATOR IN TABLE NAME",
            ZeroLengthKey => "ASCII SEPARATOR IN KEY NAME",
            AsciiSeparatorInKeyName => "ZERO LENGTH KEY (NOT EQUIVLAENT TO NONE)",
        }.to_string()
    }
}

#[derive(Debug)]
struct DBPath {
    table: String,
    maybe_key: Option<String>
}

impl DBPath {
    fn new<'a, T>(raw_table: T, maybe_raw_key: Option<T>) -> Result<Self, DBError> where T: AsRef<str> {
        let table: String = raw_table.as_ref().nfc().collect();
        let maybe_key: Option<String> = maybe_raw_key.map(|raw_key| raw_key.as_ref().nfc().collect() );

        let separators: &[char] = SEPARATORS.as_ref();

        let dirty_key = maybe_key.as_ref().map(|key| key.contains(separators)).unwrap_or(false);
        let some_empty_key = maybe_key.as_ref().map(|key| key.len() < 1).unwrap_or(false);

        if table.len() < 1 {
            Err(ZeroLengthTable)
        } else if table.contains(separators) {
            Err(AsciiSeparatorInTableName)
        } else if some_empty_key {
            Err(ZeroLengthKey)
        } else if dirty_key {
            Err(AsciiSeparatorInKeyName)
        } else {
            Ok(DBPath { table, maybe_key })
        }
    }

}

impl ToString for DBPath {
    fn to_string(&self) -> String {
        let mut raw = self.table.clone();
        raw.push(RECORD_SEP);
        if let Some(key) = self.maybe_key.as_ref() {
            raw.push_str(key);
        };
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::DBError::*;
    use super::DBPath;
    use super::*;
    use std::char::from_u32_unchecked;

    #[test]
    fn invalidate_bad_strings() {
        let bad_table_record = DBPath::new(format!("as{}df", RECORD_SEP), None);
        let bad_table_unit = DBPath::new(format!("as{}df", UNIT_SEP), None);
        let bad_key_record = DBPath::new("this", Some(format!("as{}df", RECORD_SEP).as_ref()));
        let bad_key_unit = DBPath::new("this", Some(format!("as{}df", UNIT_SEP).as_ref()));
        let empty_table = DBPath::new("", None);
        let empty_key = DBPath::new("this", Some(""));
        assert!(bad_table_record.unwrap_err() == AsciiSeparatorInTableName);
        assert!(bad_table_unit.unwrap_err() == AsciiSeparatorInTableName);
        assert!(bad_key_record.unwrap_err() == AsciiSeparatorInKeyName);
        assert!(bad_key_unit.unwrap_err() == AsciiSeparatorInKeyName);
        assert!(empty_table.unwrap_err() == ZeroLengthTable);
        assert!(empty_key.unwrap_err() == ZeroLengthKey);
    }

    #[test]
    fn unicode_successor_function() {
        assert!(unicode_successor('a').unwrap() == 'b');
        assert!(unicode_successor(from_u32(0xF66).unwrap()).unwrap() == from_u32(0xF67).unwrap());
        assert!(unicode_successor(from_u32(0xD7FE).unwrap()).unwrap() == from_u32(0xD7FF).unwrap());
        assert!(unicode_successor(from_u32(0xD7FF).unwrap()).unwrap() == from_u32(0xE000).unwrap());
        assert!(unicode_successor(from_u32(0xE000).unwrap()).unwrap() == from_u32(0xE001).unwrap());
        assert!(unicode_successor(from_u32(0xFAD9).unwrap()).unwrap() == from_u32(0xFADA).unwrap());
        assert!(unicode_successor(from_u32(UNICODE_LIMIT - 1).unwrap()).unwrap() == from_u32(UNICODE_LIMIT).unwrap());

        unsafe {
            assert!(unicode_successor(from_u32_unchecked(0xD8AA)).is_none());
        }
        assert!(unicode_successor(from_u32(UNICODE_LIMIT).unwrap()).is_none());
    }

    #[test]
    fn string_successor_function() {
        assert!(string_successor("foo".to_string()).unwrap() == "fop".to_string());
        let across_the_surrogates: String = "foo\u{D7FF}".to_string();
        assert!(string_successor(across_the_surrogates).unwrap() == "foo\u{E000}".to_string());
        let farthest_ending: String = "foo\u{10FFFF}".to_string();
        assert!(string_successor(dbg!(farthest_ending)).is_none());
    }
}
