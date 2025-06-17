use dbus::arg::{RefArg, Variant};

pub fn make_key_value_pairs(value: &Variant<Box<dyn RefArg>>) -> Vec<(String, &dyn RefArg)> {
    let mut pairs = Vec::new();

    if let Some(iter) = value.0.as_iter() {
        // Every odd entry is a key, every even entry is a value
        let mut enumerate = iter.enumerate();
        while let Some((i, entry)) = enumerate.next() {
            if i % 2 == 0 {
                if let Some(key) = entry.as_str() {
                    if let Some(value) = enumerate.next() {
                        pairs.push((key.to_string(), value.1));
                    }
                }
            }
        }
    }

    pairs
}

pub fn as_str(arg: &dyn RefArg) -> Result<String, String> {
    if let Some(s) = arg.as_str() {
        Ok(s.to_string())
    } else {
        Err(format!("ARG is not a string: {:?}", arg))
    }
}

pub fn as_i64(arg: &dyn RefArg) -> Result<i64, String> {
    if let Some(i) = arg.as_i64() {
        Ok(i)
    } else {
        Err(format!("ARG is not an int: {:?}", arg))
    }
}

pub fn as_str_vec(arg: &dyn RefArg) -> Result<Vec<String>, String> {
    if let Some(iter) = arg.as_iter() {
        // Casting arg to an iterator doesn't actually iterate over the underlying array,
        // instead it'll only have one item, which is the iterator itself.
        // We need to cast it AGAIN to actually get the underlying array's iterator
        let mut vec = Vec::new();
        for item in iter {
            if let Some(iter) = item.as_iter() {
                vec.extend(iter.map(|s| s.as_str().unwrap_or("").to_string()));
            }
        }

        Ok(vec)
    } else {
        Err(format!("ARG is not an iterable of strings: {:?}", arg))
    }
}