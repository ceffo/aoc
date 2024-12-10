#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    Ok("part1".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "";
        assert_eq!("", process(input)?);
        Ok(())
    }
}
