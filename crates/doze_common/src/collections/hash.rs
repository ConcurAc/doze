use rapidhash::{RapidHashMap, RapidHashSet};

pub type HashMap<K, V> = RapidHashMap<K, V>;
pub use rapidhash::HashMapExt;

pub type HashSet<K> = RapidHashSet<K>;
pub use rapidhash::HashSetExt;
