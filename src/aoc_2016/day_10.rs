use aoc::prelude::*;

#[cfg(test)]
mod tests {
    use aoc::prelude_test::*;

    solution_tests! {
        example {
            input = "magic chips 2 3
product outputs 0 1 2
value 5 goes to bot 2
bot 2 gives low to bot 1 and high to bot 0
value 3 goes to bot 1
bot 1 gives low to output 1 and high to bot 0
bot 0 gives low to output 2 and high to output 0
value 2 goes to bot 2";
            answers = unsigned![1, 30];
        }
        actual_answers = unsigned![157, 1085];
    }
}

/// Contains solution implementation items.
mod solution {
    use super::*;
    use aoc::parse::trim;
    use itertools::process_results;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::u8 as pnum,
        combinator::map,
        multi::{many1, separated_list1},
    };
    use std::{cmp::Ordering, collections::HashMap, marker::PhantomData};

    /// The number type to use for bot numbers, chip values, and output numbers.
    type Num = u8;

    /// A recipient to whom a bot can give a chip.
    ///
    /// Can be parsed from text input.
    #[derive(Debug, PartialEq, Eq, Hash)]
    enum Recipient {
        /// Another bot with its number.
        Bot(Num),
        /// An output with its number.
        Output(Num),
    }
    impl Parsable<'_> for Recipient {
        fn parser(input: &'_ str) -> NomParseResult<&'_ str, Self> {
            alt((
                map((tag("bot "), pnum), |(_, n)| Self::Bot(n)),
                map((tag("output "), pnum), |(_, n)| Self::Output(n)),
            ))
            .parse(input)
        }
    }
    impl PartialOrd for Recipient {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Recipient {
        fn cmp(&self, other: &Self) -> Ordering {
            match self {
                Recipient::Bot(n1) => match other {
                    Recipient::Bot(n2) => n1.cmp(n2),
                    Recipient::Output(_) => Ordering::Less,
                },
                Recipient::Output(n1) => match other {
                    Recipient::Bot(_) => Ordering::Greater,
                    Recipient::Output(n2) => n1.cmp(n2),
                },
            }
        }
    }

    /// A single instruction for the [`Factory`].
    ///
    /// Can be parsed from text input.
    #[derive(Debug, PartialEq, Eq, Hash)]
    enum Instruction {
        /// A bot is to pick a chip.
        ChipToBot {
            /// The chip value to pick up.
            chip_value: Num,
            /// The bot number of the bot to whom the chip goes.
            bot_num: Num,
        },
        /// The bot to do something with its chips.
        ///
        /// Only to be executed when a bot has two chips.
        BotGive {
            /// The bot number of the bot to give the chips.
            bot_num: Num,
            /// The recipient of the chip with the lower value.
            low_to: Recipient,
            /// The recipient of the chip with the higher value.
            high_to: Recipient,
        },
    }
    impl Parsable<'_> for Instruction {
        fn parser(input: &'_ str) -> NomParseResult<&'_ str, Self> {
            trim(
                true,
                alt((
                    map(
                        (tag("value "), pnum, tag(" goes to bot "), pnum),
                        |(_, chip_value, _, bot_num)| Self::ChipToBot {
                            chip_value,
                            bot_num,
                        },
                    ),
                    map(
                        (
                            tag("bot "),
                            pnum,
                            tag(" gives low to "),
                            Recipient::parser,
                            tag(" and high to "),
                            Recipient::parser,
                        ),
                        |(_, bot_num, _, low_to, _, high_to)| Self::BotGive {
                            bot_num,
                            low_to,
                            high_to,
                        },
                    ),
                )),
            )
            .parse(input)
        }
    }
    impl PartialOrd for Instruction {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Instruction {
        fn cmp(&self, other: &Self) -> Ordering {
            match self {
                Instruction::ChipToBot {
                    chip_value: c1,
                    bot_num: b1,
                } => match other {
                    Instruction::ChipToBot {
                        chip_value: c2,
                        bot_num: b2,
                    } => (b1, c1).cmp(&(b2, c2)),
                    Instruction::BotGive {
                        bot_num: _,
                        low_to: _,
                        high_to: _,
                    } => Ordering::Less,
                },
                Instruction::BotGive {
                    bot_num: b1,
                    low_to: l1,
                    high_to: h1,
                } => match other {
                    Instruction::ChipToBot {
                        chip_value: _,
                        bot_num: _,
                    } => Ordering::Greater,
                    Instruction::BotGive {
                        bot_num: b2,
                        low_to: l2,
                        high_to: h2,
                    } => (b1, l1, h1).cmp(&(b2, l2, h2)),
                },
            }
        }
    }

    /// Chips held by a bot.
    #[derive(Debug, PartialEq, Eq)]
    struct BotChips {
        /// The chip with the lower value.
        pub low: Num,
        /// The chip with the higher value.
        pub high: Num,
        /// This is just so we cannot create this manually.
        _phant: PhantomData<()>,
    }
    impl BotChips {
        /// Creates this given two chip values.
        ///
        /// Fails if the two chips have the same value, i.e. `a == b`.
        pub fn new(a: Num, b: Num) -> AocResult<Self> {
            if a == b {
                Err(AocError::Process(
                    format!("Cannot have chips of the same value {a}!").into(),
                ))
            } else {
                Ok(Self {
                    low: a.min(b),
                    high: a.max(b),
                    _phant: PhantomData,
                })
            }
        }
    }

    /// A bot that can hold chips.
    #[derive(Clone, Debug, Default)]
    struct Bot {
        /// The chips currently held by the bot.
        ///
        /// This will never have more than two elements.
        chips: Vec<Num>,
    }
    impl Bot {
        /// Adds a chip to the bot if possible and returns whether or not the
        /// bot could accept the chip.
        ///
        /// A return value of `false` means that the bot already has two chips.
        pub fn add_chip(&mut self, value: Num) -> bool {
            (self.chips.len() < 2).and_do(|| self.chips.push(value))
        }

        /// Returns the chips the bot is holding, or `None` if it does not
        /// have two chips.
        ///
        /// Fails if the bot somehow ended up holding two of the same chip.
        pub fn chips(&self) -> AocResult<Option<BotChips>> {
            Ok(if self.chips.len() == 2 {
                Some(BotChips::new(self.chips[0], self.chips[1])?)
            } else {
                None
            })
        }

        /// Removes all chips from the bot.
        pub fn reset_chips(&mut self) {
            self.chips.clear();
        }
    }

    /// The current state of the entire factory.
    #[derive(Clone, Debug, Default)]
    struct Factory {
        /// The bots that have currently been instructed to do something based
        /// on their number.
        bots: HashMap<Num, Bot>,
        /// The chip values in each output that has a chip, based on output
        /// number.
        outputs: HashMap<Num, Num>,
    }
    impl Factory {
        /// Returns a mutable reference to bot `n`, adding the bot if it does
        /// not exist.
        fn get_bot(&mut self, n: Num) -> &mut Bot {
            self.bots.entry(n).or_default()
        }

        /// Returns the value of the chip in output `n` or `None` if the output
        /// has no chip.
        fn get_output_value(&self, n: Num) -> Option<Num> {
            self.outputs.get(&n).copied()
        }

        /// Returns the bot number of the bot that is currently holding the
        /// `chips` or `None` if not bot is holding them.
        ///
        /// Fails if an invalid state is detected.
        pub fn bot_with_chips(&self, chips: &BotChips) -> AocResult<Option<Num>> {
            for (bot_num, bot) in self.bots.iter() {
                if let Some(cs) = bot.chips()?
                    && cs == *chips
                {
                    return Ok(Some(*bot_num));
                }
            }
            Ok(None)
        }

        /// Executes a single instruction and returns whether or not it could be
        /// executed right now.
        ///
        /// Fails if an invalid state is detected.
        pub fn execute_instruction(&mut self, instruction: &Instruction) -> AocResult<bool> {
            Ok(match instruction {
                Instruction::ChipToBot {
                    chip_value,
                    bot_num,
                } => {
                    let bot = self.get_bot(*bot_num);
                    bot.add_chip(*chip_value)
                }
                Instruction::BotGive {
                    bot_num,
                    low_to,
                    high_to,
                } => {
                    let bot = self.get_bot(*bot_num);
                    match bot.chips()? {
                        Some(chips) => {
                            // We can only complete this if both transactions can be completed.
                            let mut new_factory = self.clone();

                            process_results(
                                [low_to, high_to]
                                    .into_iter()
                                    .zip([chips.low, chips.high])
                                    .map(|(recip, cv)| {
                                        Ok(match recip {
                                            Recipient::Bot(give_to) => {
                                                if bot_num == give_to {
                                                    return Err(AocError::Process(format!("Bot {bot_num} is trying to give a chip to itself!").into()))
                                                }
                                                new_factory.get_bot(*give_to).add_chip(cv)
                                            }

                                            Recipient::Output(output_num) => {
                                                if let Some(v) = new_factory.outputs.get(output_num) {
                                                    return Err(AocError::Process(format!("Output {output_num} already contains a chip of value {v}").into()))
                                                }
                                                new_factory.outputs.insert(*output_num, cv);
                                                true
                                            }
                                        })
                                    }),
                                |mut iter| iter.all(std::convert::identity),
                            )?.and_do(|| {
                                // Everything passed so commit
                                new_factory.get_bot(*bot_num).reset_chips();
                                *self = new_factory;
                            })
                        }
                        None => false,
                    }
                }
            })
        }
    }

    /// The end result of the [`Factory`] after all instructions have been
    /// executed.
    pub struct FactoryOutput {
        /// The bot that held the magic chips.
        pub magic_bot: Num,
        /// The product of the specified outputs.
        pub output_product: u64,
    }

    /// A set of instructions for the [`Factory`].
    ///
    /// Can be parsed from text input.
    #[derive(Debug)]
    pub struct InstructionSet {
        /// Which magic chips that need to be held by a bot to determine the
        /// [`FactoryOutput::magic_bot`].
        magic_chips: BotChips,
        /// Which outputs to use when calculating the
        /// [`FactoryOutput::output_product`].
        product_outputs: Vec<Num>,
        /// The set of instructions.
        set: Vec<Instruction>,
    }
    impl Parsable<'_> for InstructionSet {
        fn parser(input: &'_ str) -> NomParseResult<&'_ str, Self> {
            map(
                (
                    trim(true, (tag("magic chips "), pnum, tag(" "), pnum)),
                    trim(
                        true,
                        (tag("product outputs "), separated_list1(tag(" "), pnum)),
                    ),
                    many1(Instruction::parser),
                ),
                |((_, a, _, b), (_, product_outputs), set)| Self {
                    magic_chips: BotChips::new(a, b).unwrap(),
                    product_outputs,
                    set,
                },
            )
            .parse(input)
        }
    }
    impl InstructionSet {
        /// Executes all instructions and returns the resulting
        /// [`FactoryOutput`].
        ///
        /// Fails if the instructions are invalid, could not all be executed, or
        /// if there was no solution in terms of all fields of the
        /// [`FactoryOutput`].
        pub fn execute(mut self) -> AocResult<FactoryOutput> {
            let mut factory = Factory::default();
            let mut magic_bot = None;

            // Go through the instructions until none are left to execute.
            while !self.set.is_empty() {
                let mut error = Ok(());
                let mut inst_executed = false;

                // Try to execute all instructions and remove those that executed
                self.set.retain(|inst| {
                    try {
                        let executed = factory.execute_instruction(inst)?;
                        if executed {
                            inst_executed = true;

                            // Check for the magic bot
                            if let Some(n) = factory.bot_with_chips(&self.magic_chips)? {
                                magic_bot = Some(n);
                            }
                        }
                        !executed
                    }
                    .unwrap_or_else(|e| {
                        error = Err(e);
                        true
                    })
                });

                // Did an error occur?
                error?;

                // No instruction can executed, so we cannot finish
                inst_executed.ok_or(AocError::NoSolution)?;
            }

            Ok(FactoryOutput {
                magic_bot: magic_bot.ok_or(AocError::NoSolution)?,
                output_product: process_results(
                    self.product_outputs.iter().map(|on| {
                        factory
                            .get_output_value(*on)
                            .map(u64::from)
                            .ok_or(AocError::NoSolution)
                    }),
                    |output_values| output_values.product(),
                )?,
            })
        }
    }
}

use solution::*;

/// Solution struct.
pub const SOLUTION: Solution = Solution {
    day: 10,
    name: "Balance Bots",
    preprocessor: Some(|input| Ok(Box::new(InstructionSet::from_str(input)?.execute()?).into())),
    solvers: &[
        // Part one
        |input| Ok(input.expect_data::<FactoryOutput>()?.magic_bot.into()),
        // Part two
        |input| Ok(input.expect_data::<FactoryOutput>()?.output_product.into()),
    ],
};
