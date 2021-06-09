use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseProblem {
    object: &'static str,
    string: Option<String>,
}

impl ParseProblem {
    pub fn new(object: &'static str, string: Option<String>) -> ParseProblem {
        ParseProblem { object, string }
    }

}

#[derive(Debug, Clone)]
pub enum AocError {
    Parse(ParseProblem),
}

impl From<ParseProblem> for AocError {
    fn from(pp: ParseProblem) -> AocError {
        AocError::Parse(pp)
    }
}

impl fmt::Display for AocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
               match self {
                   AocError::Parse(p) => {
                       let s = match p.string {
                           Some(s) => format!(" from {}", s),
                           None => 
                               
                               
                               format!("Could not parse {}{}", p.object,
                               }
                       ),
                   }
               )
        }
    }

    impl Error for AocError {}
