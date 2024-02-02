pub mod response {
    use std::fmt::Display;
    use std::str::FromStr;
    #[derive(Debug)]
    pub enum Status {
        Ok,
        Created,
        Accepted,
        BadRequest,
        NotFound,
        InternalServerError,
    }

    impl Status {
        pub fn status_code(&self) -> i32 {
            return match self {
                Self::Ok => 200,
                Self::Created => 201,
                Self::Accepted => 202,
                Self::BadRequest => 400,
                Self::NotFound => 404,
                Self::InternalServerError => 500,
            };
        }
        pub fn response_string(&self) -> String {
            return format!("HTTP/1.1 {} {}", self.status_code(), self);
        }
    }

    impl Display for Status {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let as_string = format!("{:?}", self);

            let mut result: Vec<char> = Vec::new();

            for (i, char) in as_string.chars().enumerate() {
                if char.is_uppercase() && i != 0 {
                    result.push(char::from_str(" ").unwrap());
                    result.push(char);
                    continue;
                }
                result.push(char)
            }

            let as_string: String = result.iter().collect();

            write!(f, "{}", as_string.to_uppercase()).expect("it to write yk");
            return Ok(());
        }
    }
}

