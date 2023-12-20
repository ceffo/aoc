use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet},
    fmt::{self, Display, Formatter},
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, space1},
    combinator::{map, map_res, value},
    multi::{many_m_n, separated_list0},
    sequence::{pair, preceded},
    Err, IResult,
};

use crate::custom_error::AocError;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum Card {
    J,
    N(u8),
    T,
    Q,
    K,
    A,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Card::N(n) => write!(f, "{}", n),
            Card::T => write!(f, "T"),
            Card::J => write!(f, "J"),
            Card::Q => write!(f, "Q"),
            Card::K => write!(f, "K"),
            Card::A => write!(f, "A"),
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Card::N(n) => write!(f, "{}", n),
            Card::T => write!(f, "T"),
            Card::J => write!(f, "J"),
            Card::Q => write!(f, "Q"),
            Card::K => write!(f, "K"),
            Card::A => write!(f, "A"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum HandType {
    HighCard,
    OnePair,
    TwoPairs,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Hand {
    cards: [Card; 5],
    hand_type: Option<HandType>,
}

impl Hand {
    fn new(cards: [Card; 5]) -> Self {
        Self {
            cards,
            hand_type: None,
        }
    }

    fn get_resolved(&self) -> Self {
        let mut hand = *self;
        hand.resolve_hand_type();
        hand
    }

    fn resolve_hand_type(&mut self) {
        if self.hand_type.is_some() {
            return;
        }
        let hand_type = get_type(&self.cards).unwrap();
        self.hand_type = Some(hand_type);
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for card in self.cards.iter() {
            write!(f, "{}", card)?;
        }
        Ok(())
    }
}

fn find_joker_subsitutions(hand: &[Card; 5]) -> Vec<[Card; 5]> {
    // generate all possible hands with jokers J
    if !hand.contains(&Card::J) || hand == &[Card::J; 5] {
        // no jokers or just jokers in hand
        return vec![*hand];
    }
    let non_jokers = hand
        .iter()
        .filter(|c| **c != Card::J)
        .copied()
        .collect::<HashSet<_>>();
    non_jokers
        .iter()
        .map(|non_joker| {
            let mut hand = *hand;
            hand.iter_mut().for_each(|c| {
                if *c == Card::J {
                    *c = *non_joker;
                }
            });
            hand
        })
        .collect()
}
fn get_type(cards: &[Card; 5]) -> Result<HandType, Err<String>> {
    let hands = find_joker_subsitutions(cards);
    let hand_types = hands
        .into_iter()
        .map(|hand| get_type_simple(&hand))
        .collect::<Result<Vec<_>, _>>()?;
    hand_types
        .into_iter()
        .max()
        .ok_or(Err::Error("No max hand type found".to_string()))
}

fn get_type_simple(cards: &[Card; 5]) -> Result<HandType, Err<String>> {
    let mut counts: BTreeMap<Card, u8> = BTreeMap::new();
    // count the cards in the hand by rank
    for card in cards.iter() {
        counts.entry(*card).and_modify(increment).or_insert(1);
    }
    // collect the number of cards of each rank and sort by count
    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    let mut idx = 0;
    let mut k = Kind::One;
    let mut pattern = [Kind::One; 5];
    // build a pattern of the hand
    for (_, count) in counts.iter() {
        for _ in 0..*count {
            pattern[idx] = k;
            idx += 1;
        }
        k = match k {
            Kind::One => Kind::Two,
            Kind::Two => Kind::Three,
            Kind::Three => Kind::Four,
            Kind::Four => Kind::Five,
            Kind::Five => Kind::Five,
        }
    }
    // match the pattern to a hand type
    match pattern {
        [Kind::One, Kind::One, Kind::One, Kind::One, Kind::One] => Ok(HandType::FiveOfAKind),
        [Kind::One, Kind::One, Kind::One, Kind::One, Kind::Two] => Ok(HandType::FourOfAKind),
        [Kind::One, Kind::One, Kind::One, Kind::Two, Kind::Two] => Ok(HandType::FullHouse),
        [Kind::One, Kind::One, Kind::One, Kind::Two, Kind::Three] => Ok(HandType::ThreeOfAKind),
        [Kind::One, Kind::One, Kind::Two, Kind::Two, Kind::Three] => Ok(HandType::TwoPairs),
        [Kind::One, Kind::One, Kind::Two, Kind::Three, Kind::Four] => Ok(HandType::OnePair),
        [Kind::One, Kind::Two, Kind::Three, Kind::Four, Kind::Five] => Ok(HandType::HighCard),
        _ => Err(Err::Error("Invalid hand".to_string())),
    }
}

fn cmp_cards(a: &[Card; 5], b: &[Card; 5]) -> Ordering {
    for (a, b) in a.iter().zip(b.iter()) {
        match a.cmp(b) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.hand_type.cmp(&other.hand_type) {
            Ordering::Equal => cmp_cards(&self.cards, &other.cards),
            other => other,
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn increment(n: &mut u8) {
    *n += 1;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Kind {
    One,
    Two,
    Three,
    Four,
    Five,
}

fn parse_card(input: &str) -> IResult<&str, Card> {
    let mut parser = alt((
        value(Card::A, tag("A")),
        value(Card::K, tag("K")),
        value(Card::Q, tag("Q")),
        value(Card::J, tag("J")),
        value(Card::T, tag("T")),
        value(Card::N(9), tag("9")),
        value(Card::N(8), tag("8")),
        value(Card::N(7), tag("7")),
        value(Card::N(6), tag("6")),
        value(Card::N(5), tag("5")),
        value(Card::N(4), tag("4")),
        value(Card::N(3), tag("3")),
        value(Card::N(2), tag("2")),
        value(Card::N(1), tag("1")),
    ));
    parser(input)
}

fn parse_hand(input: &str) -> IResult<&str, Hand> {
    let (input, cards) = map_res(many_m_n(5, 5, parse_card), |c| {
        TryInto::<[Card; 5]>::try_into(c)
    })(input)?;
    Ok((input, Hand::new(cards)))
}

#[derive(Debug, PartialEq, Eq)]
struct Game {
    hands: Vec<(Hand, u32)>,
}

impl Game {
    fn calculate_winnings(&self) -> u64 {
        // sort the hands
        let mut hands = self.hands.clone();
        hands.sort_by(|(a, _), (b, _)| a.cmp(b));
        hands
            .iter()
            .enumerate()
            .map(|(i, (_, bet))| (i as u64 + 1) * *bet as u64)
            .sum()
    }
}

fn parse_game(input: &str) -> IResult<&str, Game> {
    let (input, hands) = separated_list0(
        line_ending,
        pair(
            map(parse_hand, |h| h.get_resolved()),
            preceded(space1, nom::character::complete::u32),
        ),
    )(input)?;
    Ok((input, Game { hands }))
}

#[tracing::instrument]
pub fn process(input: &str) -> miette::Result<String, AocError> {
    let (_, game) = parse_game(input).map_err(|e| AocError::ParseError(e.to_string()))?;
    let winnings = game.calculate_winnings();
    Ok(winnings.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_card_ordering() {
        assert!(Card::A > Card::K);
        assert!(Card::K > Card::Q);
        assert!(Card::Q > Card::T);
        assert!(Card::T > Card::N(9));
        assert!(Card::N(9) > Card::N(8));
        assert!(Card::N(8) > Card::N(7));
        assert!(Card::N(7) > Card::N(6));
        assert!(Card::N(6) > Card::N(5));
        assert!(Card::N(5) > Card::N(4));
        assert!(Card::N(4) > Card::N(3));
        assert!(Card::N(3) > Card::N(2));
        assert!(Card::N(2) > Card::N(1));
        assert!(Card::N(1) > Card::N(0));
        assert!(Card::N(0) > Card::J);
    }

    #[rstest]
    #[case(
        Hand::new([Card::A, Card::K, Card::Q, Card::J, Card::T]
        ),
        HandType::OnePair
    )]
    #[case(
        Hand::new([Card::A, Card::A, Card::A, Card::A, Card::A]
        ),
        HandType::FiveOfAKind
    )]
    #[case(
        Hand::new([Card::A, Card::A, Card::A, Card::A, Card::K]
        ),
        HandType::FourOfAKind
    )]
    #[case(
        Hand::new([Card::A, Card::A, Card::A, Card::K, Card::K]
        ),
        HandType::FullHouse
    )]
    #[case(
        Hand::new([Card::A, Card::A, Card::A, Card::K, Card::Q]
        ),
        HandType::ThreeOfAKind
    )]
    #[case(
        Hand::new([Card::A, Card::A, Card::K, Card::K, Card::Q]
        ),
        HandType::TwoPairs
    )]
    #[case(
        Hand::new([Card::A, Card::A, Card::K, Card::Q, Card::J]
        ),
        HandType::ThreeOfAKind
    )]
    #[case(
        Hand::new([Card::A, Card::K, Card::Q, Card::N(1), Card::T]
        ),
        HandType::HighCard
    )]
    fn test_hand_type(#[case] hand: Hand, #[case] expected: HandType) {
        assert_eq!(hand.get_resolved().hand_type.unwrap(), expected);
    }

    #[rstest]
    #[case("AAAAA", Hand::new([Card::A, Card::A, Card::A, Card::A, Card::A]))]
    #[case("AKQJT", Hand::new([Card::A, Card::K, Card::Q, Card::J, Card::T]))]
    #[case("A29T1", Hand::new([Card::A, Card::N(2), Card::N(9), Card::T, Card::N(1)]))]
    fn test_parse_hand(#[case] input: &str, #[case] expected: Hand) {
        assert_eq!(parse_hand(input).unwrap(), ("", expected));
    }

    #[rstest]
    #[case(
        "AAAAA 1",
        Game {
            hands: vec![(Hand::new([Card::A, Card::A, Card::A, Card::A, Card::A] ).get_resolved(), 1)]
        }
    )]
    #[case(
        "AKQJT 2343\nA2QT5 123",
        Game {
            hands: vec![
                (Hand::new([Card::A, Card::K, Card::Q, Card::J, Card::T] ).get_resolved(), 2343),
                (Hand::new([Card::A, Card::N(2), Card::Q, Card::T, Card::N(5)] ).get_resolved(), 123),
            ]
        }
    )]
    fn test_parse_game(#[case] input: &str, #[case] expected: Game) {
        assert_eq!(parse_game(input).unwrap(), ("", expected));
    }

    #[test]
    fn test_process() -> miette::Result<()> {
        let input = "32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";
        assert_eq!("5905", process(input)?);
        Ok(())
    }
}
