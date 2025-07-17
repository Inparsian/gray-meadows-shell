use dbus::arg::{RefArg, Variant};

pub fn make_key_value_pairs(value: &Variant<Box<dyn RefArg>>) -> Vec<(String, &dyn RefArg)> {
    let mut pairs = Vec::new();

    if let Some(iter) = value.0.as_iter() {
        // Every odd entry is a key, every even entry is a value
        let mut enumerate = iter.enumerate();
        while let Some((i, entry)) = enumerate.next() {
            if i % 2 == 0 {
                if let (Some(key), Some(value)) = (entry.as_str(), enumerate.next()) {
                    pairs.push((key.to_owned(), value.1));
                }
            }
        }
    }

    pairs
}

pub fn as_str_vec(arg: &dyn RefArg) -> Result<Vec<String>, String> {
    arg.as_iter().map_or(Err(format!("arg is not a String iterable: {:?}", arg)), |iter| {
        // Casting arg to an iterator doesn't actually iterate over the underlying array,
        // instead it'll only have one item, which is the iterator itself.
        // We need to cast it AGAIN to actually get the underlying array's iterator
        let mut vec = Vec::new();
        for item in iter {
            if let Some(iter) = item.as_iter() {
                vec.extend(iter.map(|s| s.as_str().unwrap_or("").to_owned()));
            }
        }

        Ok(vec)
    })
}