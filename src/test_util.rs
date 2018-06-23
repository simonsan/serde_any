use error::Error;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Wizard {
    pub name: String,
    pub is_late: bool,
    pub color: String,
    pub age: u32,
    pub friends: Vec<String>,
}

pub fn radagast() -> Wizard {
    Wizard {
        name: "Radagast".to_string(),
        color: "Brown".to_string(),
        is_late: true,
        age: 8000,
        friends: vec!["animals".to_string()],
    }
}

pub fn gandalf_the_white() -> Wizard {
    Wizard {
        name: "Gandalf".to_string(),
        color: "White".to_string(),
        is_late: false,
        age: 9001,
        friends: vec![
            "hobbits".to_string(),
            "dwarves".to_string(),
            "elves".to_string(),
            "men".to_string(),
        ],
    }
}

pub fn gandalf_the_grey() -> Wizard {
    Wizard {
        name: "Gandalf".to_string(),
        color: "Grey".to_string(),
        is_late: false,
        age: 9000,
        friends: vec![
            "hobbits".to_string(),
            "dwarves".to_string(),
            "elves".to_string(),
            "men".to_string(),
        ],
    }
}

macro_rules! assert_pattern {
    ($value:expr, $pattern:pat, $message:expr) => {
        match $value {
            $pattern => {},
            r => panic!("Expected {}, got result {:?}", $message, r),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_assert_pattern_success() {
        let e: Result<(), Error> = Err(Error::NoSuccessfulParse);
        assert_pattern!(e, Err(Error::NoSuccessfulParse), "Error::NoSuccessfulParse");
    }

    #[test]
    #[should_panic]
    fn test_assert_pattern_panic() {
        let e: Result<(), Error> = Ok(());
        assert_pattern!(e, Err(Error::NoSuccessfulParse), "Error::NoSuccessfulParse");
    }
}
