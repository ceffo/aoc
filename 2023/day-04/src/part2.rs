use std::{cmp::min, collections::HashSet};

use nom::{
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::map,
    multi::{many1, separated_list0},
    sequence::preceded,
    IResult,
};

use crate::custom_error::AocError;

#[derive(Debug, PartialEq, Eq)]
struct Card {
    winning: HashSet<u32>,
    have: HashSet<u32>,
}

impl Card {
    fn new(winning: HashSet<u32>, have: HashSet<u32>) -> Self {
        Self { winning, have }
    }

    fn num_winning_numbers(&self) -> u32 {
        self.winning.intersection(&self.have).count() as u32
    }
}

type Deck = Vec<Card>;

// process_deck takes a deck of cards and counts the number of winning scratchcards
fn process_deck(deck: Deck) -> u32 {
    let num_cards = deck.len();
    deck.into_iter()
        .enumerate()
        .fold(vec![1u32; num_cards], |mut copies, (index, card)| {
            let num_winning_numbers = card.num_winning_numbers();
            if num_winning_numbers > 0 {
                // increment the number of copies of next n cards by the number of copies of this card
                for i in index + 1..min(index + 1 + num_winning_numbers as usize, copies.len()) {
                    copies[i] += copies[index];
                }
            }
            copies
        })
        .into_iter()
        .sum()
}

fn vec_to_set<T>(vec: Vec<T>) -> HashSet<T>
where
    T: Eq + std::hash::Hash,
{
    HashSet::from_iter(vec)
}

fn parse_number(input: &str) -> IResult<&str, u32> {
    preceded(
        nom::character::complete::space0,
        nom::character::complete::u32,
    )(input)
}

fn parse_number_set(input: &str) -> IResult<&str, HashSet<u32>> {
    map(many1(parse_number), vec_to_set)(input)
}

// Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
fn parse_card(input: &str) -> IResult<&str, Card> {
    let (input, _) = tag("Card ")(input)?;
    let (input, _) = parse_number(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, winning) = parse_number_set(input)?;
    let (input, _) = tag(" |")(input)?;
    let (input, have) = parse_number_set(input)?;

    Ok((input, Card::new(winning, have)))
}

fn parse_deck(input: &str) -> IResult<&str, Deck> {
    separated_list0(line_ending, parse_card)(input)
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, deck) = parse_deck(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    Ok(process_deck(deck).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Card::new([1,2,3].into(), [].into()), 0)]
    #[case(Card::new([1,2,3].into(), [1].into()), 1)]
    #[case(Card::new([1,2,3].into(), [1,2].into()), 2)]
    #[case(Card::new([1,2,3].into(), [1,2,3].into()), 3)]
    #[case(Card::new([1,2,3,4,5,6,7].into(), [1,2,3,4].into()), 4)]
    fn test_num_winning(#[case] card: Card, #[case] expected: u32) {
        assert_eq!(expected, card.num_winning_numbers());
    }

    #[rstest]
    #[case("Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53", Card::new([41,48,83,86,17].into(), [83,86,6,31,17,9,48,53].into()))]
    fn test_parse_card(#[case] input: &str, #[case] expected: Card) {
        let (_, card) = parse_card(input).unwrap();
        assert_eq!(expected, card);
    }

    #[rstest]
    #[case(vec![Card::new([1,2,3].into(), [].into())], 1)]
    #[case(vec![Card::new([1,2,3].into(), [1].into()),], 1)]
    #[case(vec![
        Card::new([1,2,3].into(), [1,2,3].into()), // win +
        Card::new([1,2,3].into(), [1,2].into()), // win ++
        Card::new([1,2,3].into(), [1].into()), // win ++++
        Card::new([1,2,3].into(), [1].into()), // win ++++++++
        Card::new([1,2,3].into(), [].into()), // no win +++++++++ 
        ], 24)]
    fn test_process_deck(#[case] input: Deck, #[case] expected: u32) {
        assert_eq!(expected, process_deck(input));
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";
        assert_eq!("30", process(input)?);
        Ok(())
    }
}
