#![allow(non_camel_case_types)]
#![feature(is_terminal)]
#![feature(const_trait_impl)]
#![feature(exact_size_is_empty)]
// just for cursor.is_empty()
#![feature(cursor_remaining)]

pub mod error;
pub mod storage;
pub mod codec;
pub mod mvcc;
pub mod row;
pub mod snapshot;

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1, 1);
        println!("lib test:{}", 0x21);
    }
}

