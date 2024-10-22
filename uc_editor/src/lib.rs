mod script_decompiler;
mod script_token;
mod u_class;
mod u_function;
mod u_package;
mod u_struct;
mod u_text_buffer;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
