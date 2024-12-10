use std::fmt::{self, Formatter};

use nom::{ 
    bytes::complete::tag, sequence::{delimited, separated_pair}, IResult
};

use crate::custom_error::AocError;


#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, muls) = parse(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let result = muls.iter().map(|mul| mul.eval()).sum::<u32>();
    Ok(result.to_string())
}

#[derive(PartialEq)]
struct Mul((u32, u32));

impl Mul {
    fn new(a: u32, b: u32) -> Self {
        Self((a, b))
    }
    fn eval(&self) -> u32 {
        self.0 .0 * self.0 .1
    }
}

impl fmt::Debug for Mul {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Mul({}, {})", self.0 .0, self.0 .1)
    }
}

#[tracing::instrument]
fn parse(input: &str) -> IResult<&str, Vec<Mul>> {
    let mut remaining = input;
    let mut result = vec![];
    while !remaining.is_empty() {
        match parse_mul(remaining) {
            Ok((input, mul)) => {
                result.push(mul);
                remaining = input;
            }
            Err(_) => {
                remaining = &remaining[1..];
            }
        }
    }
    Ok((remaining, result))
}

fn parse_mul(input: &str) -> IResult<&str, Mul> {
    let (input, (a,b)) = 
        delimited(
            tag("mul("), 
            separated_pair(
                nom::character::complete::u32, 
                tag(","), 
                nom::character::complete::u32
            ),
            tag(")"), 
        )(input)?;
    Ok((input, Mul::new(a,b)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("mul(3,4)", Ok(("", vec![Mul::new(3, 4)])))]
    #[case("mul(23,34)", Ok(("", vec![Mul::new(23, 34)])))]    
    #[case("fsd8rmul(3,4)f9834hmul(5,6)fsdhjf", Ok(("", vec![Mul::new(3, 4), Mul::new(5, 6)])))]
    fn test_parse(#[case] input: &str, #[case] expected: IResult<&str, Vec<Mul>>) {
        let actual = parse(input);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("mul(3,4)", Ok(("", Mul::new(3, 4))))]
    #[case("mul(23,34)", Ok(("", Mul::new(23, 34))))]
    fn test_parse_mul(#[case] input: &str, #[case] expected: IResult<&str, Mul>) {
        let actual = parse_mul(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))";
        assert_eq!("161", process(input)?);
        Ok(())
    }
}
