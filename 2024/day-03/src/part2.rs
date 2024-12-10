use miette::miette;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::anychar,
    combinator::{rest, value},
    multi::{many1, many_till},
    sequence::{delimited, separated_pair},
    IResult, Parser,
};

#[derive(Debug, Clone)]
pub enum InstructionParser {
    ManyTill,
    While,
}

// implement From<&str> for InstructionParser
impl From<&str> for InstructionParser {
    fn from(s: &str) -> Self {
        match s {
            "many_till" => InstructionParser::ManyTill,
            "while" => InstructionParser::While,
            _ => panic!("unknown parser"),
        }
    }
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String> {
    let (_, result) = products(input).map_err(|e| miette!("parse failed {}", e))?;
    Ok(result.to_string())
}

pub fn process2(input: &str, parser: InstructionParser) -> miette::Result<String> {
    let instr_parser = match parser {
        InstructionParser::ManyTill => instructions_manytill,
        InstructionParser::While => instructions_while,
    };
    let (_, result) =
        products_with(input, instr_parser).map_err(|e| miette!("parse failed {}", e))?;
    Ok(result.to_string())
}

#[derive(PartialEq, Debug, Clone)]
enum Instruction {
    Mul((u32, u32)),
    Do,
    Dont,
}

#[derive(PartialEq, Debug, Clone)]
enum ShouldProcess {
    Yes,
    No,
}

fn products_with(
    input: &str,
    instr_parser: impl Fn(&str) -> IResult<&str, Vec<Instruction>>,
) -> IResult<&str, u32> {
    let (input, instructions) = instr_parser(input)?;
    let (_, result) = instructions.iter().fold(
        (ShouldProcess::Yes, 0),
        |(should_process, sum), instr| match instr {
            Instruction::Mul((a, b)) => {
                if should_process == ShouldProcess::Yes {
                    (should_process, sum + a * b)
                } else {
                    (should_process, sum)
                }
            }
            Instruction::Do => (ShouldProcess::Yes, sum),
            Instruction::Dont => (ShouldProcess::No, sum),
        },
    );
    Ok((input, result))
}

#[tracing::instrument]
fn products(input: &str) -> IResult<&str, u32> {
    let (input, instructions) = instructions_while(input)?;
    let (_, result) = instructions.iter().fold(
        (ShouldProcess::Yes, 0),
        |(should_process, sum), instr| match instr {
            Instruction::Mul((a, b)) => {
                if should_process == ShouldProcess::Yes {
                    (should_process, sum + a * b)
                } else {
                    (should_process, sum)
                }
            }
            Instruction::Do => (ShouldProcess::Yes, sum),
            Instruction::Dont => (ShouldProcess::No, sum),
        },
    );
    Ok((input, result))
}

#[allow(dead_code)]
fn instructions_manytill(input: &str) -> IResult<&str, Vec<Instruction>> {
    let (input, instructions) =
        many1(many_till(anychar, instruction).map(|(_discarded, instr)| instr))(input)?;
    // consume the rest of the input
    let (input, _) = rest(input)?;
    Ok((input, instructions))
}

#[allow(dead_code)]
fn instructions_while(input: &str) -> IResult<&str, Vec<Instruction>> {
    let mut remaining = input;
    let mut result = vec![];
    let mut enable = true;
    while !remaining.is_empty() {
        match instruction(remaining) {
            Ok((input, instr)) => {
                match instr {
                    Instruction::Mul(_) => {
                        if enable {
                            result.push(instr);
                        }
                    }
                    Instruction::Do => {
                        enable = true;
                    }
                    Instruction::Dont => {
                        enable = false;
                    }
                }
                remaining = input;
            }
            Err(_) => {
                remaining = &remaining[1..];
            }
        }
    }
    Ok((remaining, result))
}

fn instruction(input: &str) -> IResult<&str, Instruction> {
    let (input, instruction) = alt((
        mul,
        value(Instruction::Do, tag("do()")),
        value(Instruction::Dont, tag("don't()")),
    ))(input)?;
    Ok((input, instruction))
}

fn mul(input: &str) -> IResult<&str, Instruction> {
    let (input, ops) = delimited(
        tag("mul("),
        separated_pair(
            nom::character::complete::u32,
            tag(","),
            nom::character::complete::u32,
        ),
        tag(")"),
    )(input)?;
    Ok((input, Instruction::Mul(ops)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("mul(3,4)", Ok(("", 12)))]
    #[case("mul(23,34)", Ok(("", 782)))]
    #[case("fsd8rmul(3,4)f9834hmul(5,6)fsdhjf", Ok(("", 42)))]
    fn test_products(#[case] input: &str, #[case] expected: IResult<&str, u32>) {
        let actual = products(input);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case("mul(3,4)", (3, 4))]
    #[case("mul(23,34)", (23, 34))]
    fn test_mul(#[case] input: &str, #[case] expected: (u32, u32)) {
        let actual = mul(input);
        assert_eq!(Ok(("", Instruction::Mul(expected))), actual);
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))";
        assert_eq!("48", process(input)?);
        Ok(())
    }
}
