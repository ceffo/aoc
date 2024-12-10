#[tracing::instrument]
pub fn process(_input: &str) -> miette::Result<String> {
    Ok("part2".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "";
        assert_eq!("part2", process(input)?);
        Ok(())
    }
}
