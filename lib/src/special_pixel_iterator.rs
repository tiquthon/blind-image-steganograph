use crate::{CountOfLeastSignificantBits, RemainingBitsAction};

pub struct PixelChannelSeparator<'a, I: Iterator<Item = &'a u8>, const CHANNELS: usize> {
    inner: I,
    count_of_least_significant_bits: [CountOfLeastSignificantBits; CHANNELS],
    expected_pixels: Option<usize>,
    pixel_channels_returned_so_far: usize,
    remaining_bits_action: RemainingBitsAction,
    #[cfg(feature = "random")]
    rng: Option<rand::rngs::StdRng>,
    channel_index: usize,
    current_byte: Option<u8>,
    current_byte_offset: usize,
}

impl<'a, I: Iterator<Item = &'a u8>, const CHANNELS: usize> PixelChannelSeparator<'a, I, CHANNELS> {
    pub fn new<J>(
        inner: J,
        count_of_least_significant_bits: [CountOfLeastSignificantBits; CHANNELS],
        expected_pixels: Option<usize>,
        remaining_bits_action: RemainingBitsAction,
    ) -> Self
    where
        J: IntoIterator<IntoIter = I>,
    {
        let bits_per_pixel = count_of_least_significant_bits
            .into_iter()
            .map(CountOfLeastSignificantBits::bit_count)
            .sum::<usize>();
        assert!(bits_per_pixel > 0);
        Self {
            inner: inner.into_iter(),
            channel_index: 0,
            current_byte: None,
            current_byte_offset: 0,
            count_of_least_significant_bits,
            expected_pixels,
            pixel_channels_returned_so_far: 0,
            remaining_bits_action,
            #[cfg(feature = "random")]
            rng: None,
        }
    }
}

impl<'a, I: Iterator<Item = &'a u8>, const CHANNELS: usize> Iterator
    for PixelChannelSeparator<'a, I, CHANNELS>
{
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let needed_bits_for_current_channel: usize =
            self.count_of_least_significant_bits[self.channel_index].bit_count();
        let bit_mask = (2u16.pow(needed_bits_for_current_channel as u32) - 1) as u8;

        let current_byte = match self.current_byte {
            Some(current_byte) => current_byte,
            None => match self.inner.next() {
                Some(next_byte) => *next_byte,
                None => {
                    return match self.expected_pixels {
                        Some(expected_pixels) => {
                            if self.pixel_channels_returned_so_far >= (CHANNELS * expected_pixels) {
                                None
                            } else {
                                match self.remaining_bits_action {
                                    RemainingBitsAction::None => None,
                                    RemainingBitsAction::Zero => {
                                        self.channel_index = (self.channel_index + 1) % CHANNELS;
                                        self.pixel_channels_returned_so_far += 1;

                                        Some((0, bit_mask))
                                    }
                                    #[cfg(feature = "random")]
                                    RemainingBitsAction::Randomize { seed } => {
                                        self.channel_index = (self.channel_index + 1) % CHANNELS;
                                        self.pixel_channels_returned_so_far += 1;
                                        use rand::Rng;
                                        let rng = self.rng.get_or_insert_with(|| <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(seed.unwrap_or_default()));
                                        Some((rng.gen::<u8>() & bit_mask, bit_mask))
                                    }
                                }
                            }
                        }
                        None => None,
                    }
                }
            },
        };

        let usable_bits_from_current_byte = 8 - self.current_byte_offset;

        let (bits, bit_mask) = if needed_bits_for_current_channel <= usable_bits_from_current_byte {
            let bits = (current_byte
                >> (8 - needed_bits_for_current_channel - self.current_byte_offset))
                & bit_mask;
            self.current_byte_offset += needed_bits_for_current_channel;
            if self.current_byte_offset >= 8 {
                self.current_byte = self.inner.next().copied();
                self.current_byte_offset -= 8;
            } else {
                self.current_byte = Some(current_byte);
            }
            (bits, bit_mask)
        } else {
            let needed_bits_from_current_byte = usable_bits_from_current_byte;
            let needed_bits_from_next_byte =
                needed_bits_for_current_channel - usable_bits_from_current_byte;
            let bit_mask_current_byte = 2u8.pow(needed_bits_from_current_byte as u32) - 1;
            let bit_mask_next_byte = 2u8.pow(needed_bits_from_next_byte as u32) - 1;
            let mut bits = ((current_byte
                >> (8 - needed_bits_from_current_byte - self.current_byte_offset))
                & bit_mask_current_byte)
                << needed_bits_from_next_byte;
            self.current_byte = self.inner.next().copied();
            self.current_byte_offset = needed_bits_from_next_byte;
            if let Some(next_byte) = self.current_byte {
                bits |= (next_byte >> (8 - needed_bits_from_next_byte)) & bit_mask_next_byte;
            }
            (bits, bit_mask)
        };
        self.channel_index = (self.channel_index + 1) % CHANNELS;
        self.pixel_channels_returned_so_far += 1;
        Some((bits, bit_mask))
    }
}

pub trait PixelChannelSeparatorIteratorExt<'a> {
    fn separate_pixel_channel_rgb(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
        expected_pixels: Option<usize>,
        remaining_bits_action: RemainingBitsAction,
    ) -> PixelChannelSeparator<'a, <Self as IntoIterator>::IntoIter, 3>
    where
        Self: Sized + IntoIterator<Item = &'a u8>;
    fn separate_pixel_channel_rgba(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
        expected_pixels: Option<usize>,
        remaining_bits_action: RemainingBitsAction,
    ) -> PixelChannelSeparator<'a, <Self as IntoIterator>::IntoIter, 4>
    where
        Self: Sized + IntoIterator<Item = &'a u8>;
}

impl<'a, I: IntoIterator<Item = &'a u8>> PixelChannelSeparatorIteratorExt<'a> for I {
    fn separate_pixel_channel_rgb(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
        expected_pixels: Option<usize>,
        remaining_bits_action: RemainingBitsAction,
    ) -> PixelChannelSeparator<'a, <Self as IntoIterator>::IntoIter, 3>
    where
        Self: Sized + IntoIterator<Item = &'a u8>,
    {
        PixelChannelSeparator::new(
            self,
            [
                count_of_least_significant_bits_in_red,
                count_of_least_significant_bits_in_green,
                count_of_least_significant_bits_in_blue,
            ],
            expected_pixels,
            remaining_bits_action,
        )
    }

    fn separate_pixel_channel_rgba(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
        expected_pixels: Option<usize>,
        remaining_bits_action: RemainingBitsAction,
    ) -> PixelChannelSeparator<'a, <Self as IntoIterator>::IntoIter, 4>
    where
        Self: Sized + IntoIterator<Item = &'a u8>,
    {
        PixelChannelSeparator::new(
            self,
            [
                count_of_least_significant_bits_in_red,
                count_of_least_significant_bits_in_green,
                count_of_least_significant_bits_in_blue,
                count_of_least_significant_bits_in_alpha,
            ],
            expected_pixels,
            remaining_bits_action,
        )
    }
}

pub struct PixelChannelCombinator<'a, I: Iterator<Item = &'a u8>, const CHANNELS: usize> {
    inner: I,
    channel_index: usize,
    current_byte: Option<u8>,
    current_byte_offset: usize,
    count_of_least_significant_bits: [CountOfLeastSignificantBits; CHANNELS],
}

impl<'a, I: Iterator<Item = &'a u8>, const CHANNELS: usize>
    PixelChannelCombinator<'a, I, CHANNELS>
{
    pub fn new<J>(
        inner: J,
        count_of_least_significant_bits: [CountOfLeastSignificantBits; CHANNELS],
    ) -> Self
    where
        J: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: inner.into_iter(),
            channel_index: 0,
            current_byte: None,
            current_byte_offset: 0,
            count_of_least_significant_bits,
        }
    }
}

impl<'a, I: Iterator<Item = &'a u8>, const CHANNELS: usize> Iterator
    for PixelChannelCombinator<'a, I, CHANNELS>
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let mut output_byte = 0;
        let mut collected_bits = 0;
        while collected_bits < 8 {
            let current_byte = match self.current_byte {
                Some(current_byte) => current_byte,
                None => match self.inner.next() {
                    Some(next_byte) => *next_byte,
                    None => {
                        return if collected_bits == 0 {
                            None
                        } else {
                            Some(output_byte)
                        }
                    }
                },
            };

            let providing_bits_by_current_channel =
                self.count_of_least_significant_bits[self.channel_index].bit_count();
            let remaining_available_bits =
                providing_bits_by_current_channel - self.current_byte_offset;
            let needed_bits = 8 - collected_bits;

            if remaining_available_bits > 0 {
                if remaining_available_bits > needed_bits {
                    let needed_bit_mask = 2u8.pow(needed_bits as u32) - 1;
                    output_byte |= ((current_byte >> (remaining_available_bits - needed_bits))
                        & needed_bit_mask)
                        << (8 - collected_bits - needed_bits);
                    collected_bits += needed_bits;
                    self.current_byte_offset += needed_bits;
                } else {
                    let remaining_bit_mask = 2u8.pow(remaining_available_bits as u32) - 1;
                    output_byte |= (current_byte & remaining_bit_mask)
                        << (8 - collected_bits - remaining_available_bits);
                    collected_bits += remaining_available_bits;
                    self.current_byte = self.inner.next().copied();
                    self.current_byte_offset = 0;
                    self.channel_index = (self.channel_index + 1) % CHANNELS;
                }
            } else {
                self.current_byte = self.inner.next().copied();
                self.current_byte_offset = 0;
                self.channel_index = (self.channel_index + 1) % CHANNELS;
            }
        }

        Some(output_byte)
    }
}

pub trait PixelChannelCombinatorIteratorExt<'a> {
    fn combine_pixel_channel_rgb(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
    ) -> PixelChannelCombinator<'a, <Self as IntoIterator>::IntoIter, 3>
    where
        Self: Sized + IntoIterator<Item = &'a u8>;

    fn combine_pixel_channel_rgba(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
    ) -> PixelChannelCombinator<'a, <Self as IntoIterator>::IntoIter, 4>
    where
        Self: Sized + IntoIterator<Item = &'a u8>;
}

impl<'a, I: IntoIterator<Item = &'a u8>> PixelChannelCombinatorIteratorExt<'a> for I {
    fn combine_pixel_channel_rgb(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
    ) -> PixelChannelCombinator<'a, <Self as IntoIterator>::IntoIter, 3>
    where
        Self: Sized + IntoIterator<Item = &'a u8>,
    {
        PixelChannelCombinator::new(
            self,
            [
                count_of_least_significant_bits_in_red,
                count_of_least_significant_bits_in_green,
                count_of_least_significant_bits_in_blue,
            ],
        )
    }

    fn combine_pixel_channel_rgba(
        self,
        count_of_least_significant_bits_in_red: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_green: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_blue: CountOfLeastSignificantBits,
        count_of_least_significant_bits_in_alpha: CountOfLeastSignificantBits,
    ) -> PixelChannelCombinator<'a, <Self as IntoIterator>::IntoIter, 4>
    where
        Self: Sized + IntoIterator<Item = &'a u8>,
    {
        PixelChannelCombinator::new(
            self,
            [
                count_of_least_significant_bits_in_red,
                count_of_least_significant_bits_in_green,
                count_of_least_significant_bits_in_blue,
                count_of_least_significant_bits_in_alpha,
            ],
        )
    }
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_channel_separator_one_one_one_and_no_remaining() {
        // Arrange
        let bytes = [0b0_1_0_1_1_0_1_0u8];

        // Act
        let output = bytes
            .iter()
            .separate_pixel_channel_rgb(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                None,
                RemainingBitsAction::None,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            output,
            [
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
            ]
        );
    }

    #[test]
    fn test_pixel_channel_separator_one_one_one_and_remaining_none() {
        // Arrange
        let bytes = [0b0_1_0_1_1_0_1_0u8];

        // Act
        let output = bytes
            .iter()
            .separate_pixel_channel_rgb(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                Some(16),
                RemainingBitsAction::None,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            output,
            [
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
            ]
        );
    }

    #[test]
    fn test_pixel_channel_separator_one_one_one_and_remaining_zero() {
        // Arrange
        let bytes = [0b0_1_0_1_1_0_1_0u8];

        // Act
        let output = bytes
            .iter()
            .separate_pixel_channel_rgb(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                Some(4),
                RemainingBitsAction::Zero,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            output,
            [
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
            ]
        );
    }

    #[test]
    fn test_pixel_channel_separator_one_one_one_one_and_no_remaining() {
        // Arrange
        let bytes = [0b0_1_0_1_1_0_1_0u8];

        // Act
        let output = bytes
            .iter()
            .separate_pixel_channel_rgba(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                None,
                RemainingBitsAction::None,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            output,
            [
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b0000000_0u8, 0b0000000_1u8),
            ]
        );
    }

    #[test]
    fn test_pixel_channel_separator_one_two_three_and_no_remaining() {
        // Arrange
        let bytes = [0b0_10_110_1_0u8];

        // Act
        let output = bytes
            .iter()
            .separate_pixel_channel_rgb(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::Two,
                CountOfLeastSignificantBits::Three,
                None,
                RemainingBitsAction::None,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            output,
            [
                (0b0000000_0u8, 0b0000000_1u8),
                (0b000000_10u8, 0b000000_11u8),
                (0b00000_110u8, 0b00000_111u8),
                (0b0000000_1u8, 0b0000000_1u8),
                (0b000000_00u8, 0b000000_11u8),
            ]
        );
    }

    #[test]
    fn test_pixel_channel_separator_one_two_three_four_and_no_remaining() {
        // Arrange
        let bytes = [0b0_10_110_10u8];

        // Act
        let output = bytes
            .iter()
            .separate_pixel_channel_rgba(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::Two,
                CountOfLeastSignificantBits::Three,
                CountOfLeastSignificantBits::Four,
                None,
                RemainingBitsAction::None,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(
            output,
            [
                (0b0000000_0u8, 0b0000000_1u8),
                (0b000000_10u8, 0b000000_11u8),
                (0b00000_110u8, 0b00000_111u8),
                (0b0000_1000u8, 0b0000_1111u8),
            ]
        );
    }

    #[test]
    fn test_pixel_channel_combinator_one_one_one_and_no_remaining() {
        // Arrange
        let bytes = [
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_0u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_0u8,
        ];

        // Act
        let output = bytes
            .iter()
            .combine_pixel_channel_rgb(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(output, [0b0_1_0_1_1_0_1_0u8, 0b0_0_1_0_0_1_0_0u8]);
    }

    #[test]
    fn test_pixel_channel_combinator_one_one_one_one_and_no_remaining() {
        // Arrange
        let bytes = [
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_0u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_0u8,
            0b0000000_1u8,
            0b0000000_0u8,
            0b0000000_0u8,
        ];

        // Act
        let output = bytes
            .iter()
            .combine_pixel_channel_rgba(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::One,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(output, [0b0_1_0_1_1_0_1_0u8, 0b0_0_1_0_0_1_0_0u8]);
    }

    #[test]
    fn test_pixel_channel_combinator_one_two_three_and_no_remaining() {
        // Arrange
        let bytes = [
            0b0000000_0u8,
            0b000000_10u8,
            0b00000_110u8,
            0b0000000_1u8,
            0b000000_00u8,
            0b00000_000u8,
            0b0000001_0u8,
            0b000001_10u8,
            0b00000_001u8,
            0b0000000_0u8,
        ];

        // Act
        let output = bytes
            .iter()
            .combine_pixel_channel_rgb(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::Two,
                CountOfLeastSignificantBits::Three,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(output, [0b0_10_110_1_0u8, 0b0_000_0_10_0u8, 0b01_0_00000u8]);
    }

    #[test]
    fn test_pixel_channel_combinator_one_two_three_four_and_no_remaining() {
        // Arrange
        let bytes = [
            0b0000000_0u8,
            0b000000_10u8,
            0b00000_110u8,
            0b0000_1010u8,
            0b0000000_1u8,
            0b000000_00u8,
            0b00000_101u8,
            0b0000_1010u8,
        ];

        // Act
        let output = bytes
            .iter()
            .combine_pixel_channel_rgba(
                CountOfLeastSignificantBits::One,
                CountOfLeastSignificantBits::Two,
                CountOfLeastSignificantBits::Three,
                CountOfLeastSignificantBits::Four,
            )
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(output, [0b0_10_110_10u8, 0b10_1_00_101u8, 0b1010_0000u8]);
    }
}
