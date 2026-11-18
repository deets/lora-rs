#![deny(rust_2018_idioms)]
#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Channel width. Lower values increase time on air, but may be able to find clear frequencies.
pub enum Bandwidth {
    _7KHz,
    _10KHz,
    _15KHz,
    _20KHz,
    _31KHz,
    _41KHz,
    _62KHz,
    _125KHz,
    _200KHz,
    _250KHz,
    _400KHz,
    _500KHz,
    _800KHz,
    _1600KHz,
}

impl Bandwidth {
    const fn is_sx128x(self) -> bool {
        matches!(self, Self::_200KHz | Self::_400KHz | Self::_800KHz | Self::_1600KHz)
    }

    pub const fn hz(self) -> u32 {
        match self {
            Bandwidth::_7KHz => 7810u32,
            Bandwidth::_10KHz => 10420u32,
            Bandwidth::_15KHz => 15630u32,
            Bandwidth::_20KHz => 20830u32,
            Bandwidth::_31KHz => 31250u32,
            Bandwidth::_41KHz => 41670u32,
            Bandwidth::_62KHz => 62500u32,
            Bandwidth::_125KHz => 125000u32,
            Bandwidth::_200KHz => 203125u32,
            Bandwidth::_250KHz => 250000u32,
            Bandwidth::_400KHz => 406250u32,
            Bandwidth::_500KHz => 500000u32,
            Bandwidth::_800KHz => 812500u32,
            Bandwidth::_1600KHz => 1625000u32,
        }
    }
}

impl From<Bandwidth> for u32 {
    fn from(value: Bandwidth) -> Self {
        value.hz()
    }
}

#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Controls the chirp rate. Lower values are slower bandwidth (longer time on air), but more robust.
pub enum SpreadingFactor {
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
}

impl SpreadingFactor {
    pub const fn factor(self) -> u32 {
        match self {
            SpreadingFactor::_5 => 5,
            SpreadingFactor::_6 => 6,
            SpreadingFactor::_7 => 7,
            SpreadingFactor::_8 => 8,
            SpreadingFactor::_9 => 9,
            SpreadingFactor::_10 => 10,
            SpreadingFactor::_11 => 11,
            SpreadingFactor::_12 => 12,
        }
    }
}

impl From<SpreadingFactor> for u32 {
    fn from(sf: SpreadingFactor) -> Self {
        sf.factor()
    }
}

#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Controls the forward error correction. Higher values are more robust, but reduces the ratio
/// of actual data in transmissions.
pub enum CodingRate {
    _4_5,
    _4_6,
    _4_7,
    _4_8,
    /// Long interleaving, SX128x only (Table 14-49 in its datasheet)
    Li4_5,
    Li4_6,
    Li4_8,
}

impl CodingRate {
    pub const fn denom(&self) -> u32 {
        match self {
            CodingRate::_4_5 => 5,
            CodingRate::_4_6 => 6,
            CodingRate::_4_7 => 7,
            CodingRate::_4_8 => 8,
            CodingRate::Li4_5 => 5,
            CodingRate::Li4_6 => 6,
            CodingRate::Li4_8 => 8,
        }
    }
}

/// LoRa modulation parameters barring frequency
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseBandModulationParams {
    pub sf: SpreadingFactor,
    pub bw: Bandwidth,
    pub cr: CodingRate,
    /// low data rate optimization, see SX127x datasheet section 4.1.1.6
    pub ldro: bool,
    /// eagerly pre-calculated symbol duration in microseconds
    t_sym_us: u32,
}

impl BaseBandModulationParams {
    /// Create a set of parameters. Low data rate optimization is derived from
    /// `sf` and `bw`, see [`Self::low_data_rate_optimize`].
    pub const fn new(sf: SpreadingFactor, bw: Bandwidth, cr: CodingRate) -> Self {
        let t_sym_us = 2u32.pow(sf.factor()) * 1_000_000 / bw.hz();
        let ldro = Self::low_data_rate_optimize(sf, bw);
        Self { sf, bw, cr, ldro, t_sym_us }
    }

    /// Whether low data rate optimization (LDRO) is used for the given `sf` and `bw`.
    ///
    /// This is the table Semtech's reference stack uses for all of its LoRa
    /// transceivers (`ral_compute_lora_ldro` in SWL2001). The datasheets describe
    /// it as "symbol duration of 16.38 ms or longer", which is what the 62.5 kHz
    /// to 250 kHz rows follow; the narrower bandwidths turn it on for every SF
    /// instead. Both ends of a link must agree on this bit, so match the reference.
    pub const fn low_data_rate_optimize(sf: SpreadingFactor, bw: Bandwidth) -> bool {
        match bw {
            // The 2.4 GHz bandwidths (SX128x) have no low data rate optimize
            // setting; the RAL table does not list them either.
            Bandwidth::_200KHz | Bandwidth::_400KHz | Bandwidth::_800KHz | Bandwidth::_1600KHz => {
                false
            }
            Bandwidth::_500KHz => false,
            Bandwidth::_250KHz => matches!(sf, SpreadingFactor::_12),
            Bandwidth::_125KHz => matches!(sf, SpreadingFactor::_11 | SpreadingFactor::_12),
            Bandwidth::_62KHz => {
                matches!(sf, SpreadingFactor::_10 | SpreadingFactor::_11 | SpreadingFactor::_12)
            }
            Bandwidth::_41KHz => matches!(
                sf,
                SpreadingFactor::_9
                    | SpreadingFactor::_10
                    | SpreadingFactor::_11
                    | SpreadingFactor::_12
            ),
            Bandwidth::_31KHz
            | Bandwidth::_20KHz
            | Bandwidth::_15KHz
            | Bandwidth::_10KHz
            | Bandwidth::_7KHz => true,
        }
    }

    /// Convert a millisecond duration to symbols, rounding up so the resulting
    /// symbol count never represents less than the requested duration.
    pub const fn delay_in_symbols_ceil(&self, delay_in_ms: u32) -> u16 {
        let delay_us = delay_in_ms as u64 * 1_000;
        let symbols = delay_us.div_ceil(self.t_sym_us as u64);
        if symbols > u16::MAX as u64 {
            u16::MAX
        } else {
            symbols as u16
        }
    }

    /// Duration of one symbol in microseconds.
    pub const fn symbol_duration_us(&self) -> u32 {
        self.t_sym_us
    }

    pub const fn symbols_to_ms(&self, symbols: u32) -> u32 {
        (self.t_sym_us * symbols) / 1_000
    }

    /// Calculates time on air for a given payload and modulation parameters.
    /// If `preamble` is None, the whole preamble including syncword is excluded from calculation.
    pub const fn time_on_air_us(
        &self,
        preamble: Option<u8>,
        explicit_header: bool,
        len: u8,
    ) -> u32 {
        if self.bw.is_sx128x() {
            return self.sx128x_time_on_air_us(preamble, explicit_header, len);
        }

        let sf = self.sf.factor() as i32;
        let t_sym_us = self.t_sym_us;

        let cr = self.cr.denom() as i32;
        let de = if self.ldro {
            1
        } else {
            0
        };
        let h = if explicit_header {
            0
        } else {
            1
        };

        const fn div_ceil(num: i32, denom: i32) -> i32 {
            (num - 1) / denom + 1
        }

        let big_ratio = div_ceil(8 * len as i32 - 4 * sf + 28 + 16 - 20 * h, 4 * (sf - 2 * de));
        let big_ratio = if big_ratio > 0 {
            big_ratio
        } else {
            0
        };
        let payload_symb_nb = (8 + big_ratio * cr) as u32;

        match preamble {
            None => t_sym_us * payload_symb_nb,
            Some(preamble) => (4 * preamble as u32 + 17 + 4 * payload_symb_nb) * t_sym_us / 4,
        }
    }

    // SWL2001 sx128x_get_lora_time_on_air_numerator, with payload CRC enabled.
    const fn sx128x_time_on_air_us(
        self,
        preamble: Option<u8>,
        explicit_header: bool,
        len: u8,
    ) -> u32 {
        let sf = self.sf.factor();
        let coding_rate = self.cr.denom() as u32;
        let payload_bits = len as u32 * 8;
        let frame_bits = payload_bits + 16;
        let bits_per_block = 4 * if sf <= 10 {
            sf
        } else {
            sf - 2
        };
        let long_interleaving =
            matches!(self.cr, CodingRate::Li4_5 | CodingRate::Li4_6 | CodingRate::Li4_8);

        let payload_symbols = if !explicit_header && long_interleaving {
            // The first eight symbols use a different rate from the rest at SF7–10.
            let first_block_bits = if sf < 7 {
                4 * sf
            } else {
                4 * (sf - 2)
            };
            let coded_bits = frame_bits * coding_rate;
            if coded_bits <= 8 * first_block_bits {
                coded_bits.div_ceil(first_block_bits)
            } else {
                (coded_bits - 8 * first_block_bits).div_ceil(bits_per_block) + 8
            }
        } else if long_interleaving {
            let header_space = if sf < 7 {
                (sf - 5) / 2
            } else {
                (sf - 7) / 2
            } * 8;
            let header_payload_bits = if frame_bits > header_space && payload_bits < header_space {
                payload_bits
            } else {
                header_space
            };
            (frame_bits.saturating_sub(header_payload_bits) * coding_rate).div_ceil(bits_per_block)
                + 8
        } else {
            let header_bits = if explicit_header {
                20
            } else {
                0
            };
            let sf_adjustment = if sf >= 7 {
                8
            } else {
                0
            };
            let bits = (frame_bits + header_bits + sf_adjustment).saturating_sub(4 * sf);
            bits.div_ceil(bits_per_block) * coding_rate + 8
        };

        let preamble_quarter_symbols = match preamble {
            Some(length) => {
                // SetPacketParams rounds up to a representable mantissa and exponent.
                let mut mantissa = length as u32;
                let mut exponent = 0;
                while mantissa > 15 {
                    mantissa = mantissa.div_ceil(2);
                    exponent += 1;
                }
                let sync_quarter_symbols = if sf < 7 {
                    25
                } else {
                    17
                };
                4 * (mantissa << exponent) + sync_quarter_symbols
            }
            None => 0,
        };
        let quarter_symbols = 4 * payload_symbols + preamble_quarter_symbols;
        let scaled_duration = (quarter_symbols as u64) * (1 << (sf - 2)) * 1_000_000;
        (scaled_duration / self.bw.hz() as u64) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LORAWAN_OVERHEAD: u8 = 13;
    // the shortest t_sym
    const SF5BW500: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_5, Bandwidth::_500KHz, CodingRate::_4_5);

    // EU868 DR6
    const SF7BW250: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_7, Bandwidth::_250KHz, CodingRate::_4_5);
    // EU868 DR5
    const SF7BW125: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_7, Bandwidth::_125KHz, CodingRate::_4_5);
    // EU868 DR4
    const SF8BW125: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_8, Bandwidth::_125KHz, CodingRate::_4_5);
    // EU868 DR3
    const SF9BW125: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_9, Bandwidth::_125KHz, CodingRate::_4_5);
    // EU868 DR2
    const SF10BW125: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_10, Bandwidth::_125KHz, CodingRate::_4_5);
    // EU868 DR1
    const SF11BW125: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_11, Bandwidth::_125KHz, CodingRate::_4_5);
    // EU868 DR0
    const SF12BW125: BaseBandModulationParams =
        BaseBandModulationParams::new(SpreadingFactor::_12, Bandwidth::_125KHz, CodingRate::_4_5);

    fn lorawan_airtime_us(params: &BaseBandModulationParams, app_payload_length: u8) -> u32 {
        params.time_on_air_us(Some(8), true, LORAWAN_OVERHEAD + app_payload_length)
    }

    // data for time-on-air tests is verified against:
    // * https://www.thethingsnetwork.org/airtime-calculator
    // * https://avbentem.github.io/airtime-calculator/ttn/

    #[test]
    fn sx128x_airtime_matches_swl2001() {
        let sf12 = BaseBandModulationParams::new(
            SpreadingFactor::_12,
            Bandwidth::_200KHz,
            CodingRate::_4_5,
        );
        assert_eq!(sf12.symbol_duration_us(), 20_164);
        assert_eq!(sf12.time_on_air_us(Some(8), true, 255), 5_550_395);

        let sf7 = BaseBandModulationParams::new(
            SpreadingFactor::_7,
            Bandwidth::_400KHz,
            CodingRate::Li4_5,
        );
        assert_eq!(sf7.time_on_air_us(Some(8), true, 10), 12_051);
        assert_eq!(sf7.time_on_air_us(Some(8), false, 10), 10_161);
    }

    #[test]
    fn time_on_air_for_short_messages() {
        assert_eq!(1152, SF5BW500.time_on_air_us(None, true, 0));
        assert_eq!(6656, SF7BW250.time_on_air_us(None, true, 0));
        assert_eq!(13312, SF7BW125.time_on_air_us(None, true, 0));
    }

    #[test]
    fn ldro_matches_semtech_table() {
        use Bandwidth::*;
        use SpreadingFactor::*;
        let bws =
            [_7KHz, _10KHz, _15KHz, _20KHz, _31KHz, _41KHz, _62KHz, _125KHz, _250KHz, _500KHz];
        // rows are SF5..SF12, columns follow `bws`
        let expected = [
            (_5, [1, 1, 1, 1, 1, 0, 0, 0, 0, 0]),
            (_6, [1, 1, 1, 1, 1, 0, 0, 0, 0, 0]),
            (_7, [1, 1, 1, 1, 1, 0, 0, 0, 0, 0]),
            (_8, [1, 1, 1, 1, 1, 0, 0, 0, 0, 0]),
            (_9, [1, 1, 1, 1, 1, 1, 0, 0, 0, 0]),
            (_10, [1, 1, 1, 1, 1, 1, 1, 0, 0, 0]),
            (_11, [1, 1, 1, 1, 1, 1, 1, 1, 0, 0]),
            (_12, [1, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        ];
        for (sf, row) in expected {
            for (bw, want) in bws.iter().zip(row) {
                let params = BaseBandModulationParams::new(sf, *bw, CodingRate::_4_5);
                assert_eq!(params.ldro, want == 1, "ldro for {sf:?}/{bw:?}");
                assert_eq!(BaseBandModulationParams::low_data_rate_optimize(sf, *bw), want == 1);
            }
        }
    }

    #[test]
    fn ldro_on_the_16_384ms_diagonal() {
        // Every cell on this diagonal has the same symbol duration (2^sf / bw
        // with sf + log2(500 kHz / bw) = 13), so LDRO must not depend on how
        // the bandwidth constants are rounded.
        use Bandwidth::*;
        use SpreadingFactor::*;
        for (sf, bw) in
            [(_7, _7KHz), (_8, _15KHz), (_9, _31KHz), (_10, _62KHz), (_11, _125KHz), (_12, _250KHz)]
        {
            let params = BaseBandModulationParams::new(sf, bw, CodingRate::_4_5);
            assert!(params.ldro, "ldro for {sf:?}/{bw:?}");
            assert!(params.symbol_duration_us().abs_diff(16_384) <= 8, "{sf:?}/{bw:?}");
        }
        assert_eq!(SF11BW125.symbol_duration_us(), 16_384);
    }

    #[test]
    fn delay_in_symbols_ceil_never_shortens_the_delay() {
        assert_eq!(1, SF7BW125.delay_in_symbols_ceil(1));
        assert_eq!(2, SF7BW125.delay_in_symbols_ceil(2));
        assert_eq!(u16::MAX, SF5BW500.delay_in_symbols_ceil(u32::MAX));
    }

    #[test]
    fn time_on_air() {
        let length = 25;
        assert_eq!(41_088, lorawan_airtime_us(&SF7BW250, length));
        assert_eq!(82_176, lorawan_airtime_us(&SF7BW125, length));
        assert_eq!(143_872, lorawan_airtime_us(&SF8BW125, length));
        assert_eq!(267_264, lorawan_airtime_us(&SF9BW125, length));
        assert_eq!(493_568, lorawan_airtime_us(&SF10BW125, length));
        assert_eq!(1_069_056, lorawan_airtime_us(&SF11BW125, length));
        assert_eq!(1_974_272, lorawan_airtime_us(&SF12BW125, length));

        let length = 26;
        assert_eq!(41_088, lorawan_airtime_us(&SF7BW250, length));
        assert_eq!(82_176, lorawan_airtime_us(&SF7BW125, length));
        assert_eq!(154_112, lorawan_airtime_us(&SF8BW125, length));
        assert_eq!(267_264, lorawan_airtime_us(&SF9BW125, length));
        assert_eq!(493_568, lorawan_airtime_us(&SF10BW125, length));
        assert_eq!(1_069_056, lorawan_airtime_us(&SF11BW125, length));
        assert_eq!(1_974_272, lorawan_airtime_us(&SF12BW125, length));

        let length = 27;
        assert_eq!(41_088, lorawan_airtime_us(&SF7BW250, length));
        assert_eq!(82_176, lorawan_airtime_us(&SF7BW125, length));
        assert_eq!(154_112, lorawan_airtime_us(&SF8BW125, length));
        assert_eq!(287_744, lorawan_airtime_us(&SF9BW125, length));
        assert_eq!(534_528, lorawan_airtime_us(&SF10BW125, length));
        assert_eq!(1_069_056, lorawan_airtime_us(&SF11BW125, length));
        assert_eq!(1_974_272, lorawan_airtime_us(&SF12BW125, length));

        let length = 28;
        assert_eq!(43_648, lorawan_airtime_us(&SF7BW250, length));
        assert_eq!(87_296, lorawan_airtime_us(&SF7BW125, length));
        assert_eq!(154_112, lorawan_airtime_us(&SF8BW125, length));
        assert_eq!(287_744, lorawan_airtime_us(&SF9BW125, length));
        assert_eq!(534_528, lorawan_airtime_us(&SF10BW125, length));
        assert_eq!(1_150_976, lorawan_airtime_us(&SF11BW125, length));
        assert_eq!(2_138_112, lorawan_airtime_us(&SF12BW125, length));
    }
}
