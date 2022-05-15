#![feature(test)]

pub mod old_proto;
pub mod v1;
// pub mod proto3;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
