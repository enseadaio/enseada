use std::cmp::Ordering;

use crate::parser::Item;

pub fn cmp_integer(i: &u64, other: &Item) -> Ordering {
    match other {
        Item::Integer(o) => i.cmp(o),
        Item::String(_) => Ordering::Greater,
        Item::List(_) => Ordering::Greater,
        Item::Null => {
            if *i == 0 {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
    }
}

pub fn cmp_string(s: &str, other: &Item) -> Ordering {
    match other {
        Item::Integer(_) => Ordering::Less,
        Item::String(other_s) => cmp_strings(s, other_s),
        Item::List(_) => Ordering::Less,
        Item::Null => cmp_strings(s, ""),
    }
}

fn cmp_strings(s: &str, other: &str) -> Ordering {
    let m1 = &map_str_to_marker(s);
    let m2 = &map_str_to_marker(other);
    m1.cmp(m2)
}

fn map_str_to_marker(s: &str) -> String {
    match s {
        "alpha" | "a" => "1".to_string(),
        "beta" | "b" => "2".to_string(),
        "milestone" | "m" => "3".to_string(),
        "rc" | "cr" => "4".to_string(),
        "snapshot" => "5".to_string(),
        "" | "ga" | "final" => "6".to_string(),
        s => format!("7-{}", s),
    }
}

pub fn cmp_list(list: &[Item], other: &Item) -> Ordering {
    match other {
        Item::Integer(_) => Ordering::Less,
        Item::String(_) => Ordering::Greater,
        Item::List(other_list) => cmp_lists(list, other_list),
        Item::Null => cmp_list(list, &Item::List(Vec::new())),
    }
}

fn cmp_lists(list: &[Item], other: &[Item]) -> Ordering {
    if other.len() > list.len() {
        return cmp_lists(other, list).reverse();
    }
    let item_count = list.len();
    for i in 0..item_count {
        let first = list.get(i).unwrap();
        let second = other.get(i).unwrap_or_else(|| &Item::Null);
        let ord = first.cmp(second);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    Ordering::Equal
}

pub fn cmp_null(other: &Item) -> Ordering {
    match other {
        Item::Integer(i) => cmp_integer(i, &Item::Null).reverse(),
        Item::String(s) => cmp_string(s, &Item::Null).reverse(),
        Item::List(list) => cmp_list(list, &Item::Null).reverse(),
        Item::Null => panic!("cannot compare two Null version items!"),
    }
}
