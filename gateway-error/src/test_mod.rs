#[cfg(test)]
mod tests {
    use crate::{Error, ErrorType};

    #[test]
    fn test_generate_error_withcause() {
        let e1 = Error::new(ErrorType::ConnectRefused);
        let mut e2 = Error::new(ErrorType::new_custom("fkall"));
        e2.because(e1);

        println!("{}", e2);

        assert_ne!(1, 2);
    }

    #[test]
    fn test_generate_error_with_context() {
        let mut e1 = Error::new(ErrorType::ConnectRefused);
        let mut e2 = Error::new(ErrorType::new_custom("fkall"));
        e1.descripe_error("mistake..");
        e2.because(e1);

        println!("{}", e2);

        assert_ne!(1, 2);
    }
}
