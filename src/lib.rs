#![allow(non_camel_case_types)]
#![feature(is_terminal)]
#![feature(const_trait_impl)]
#![feature(exact_size_is_empty)]

mod log;
mod error;
mod storage;

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        assert_eq!(1, 1);
        println!("lib test:{}", 0x21);
    }
}

