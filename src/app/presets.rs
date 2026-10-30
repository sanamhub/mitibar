//! The bar's line presets (WR-08 AC3). A preset is the two lines and both weights, applied as a
//! whole. Pure data: the Settings window only passes the index of the pill that was pressed.

use crate::app::bar::{BarField, DEFAULT_BOTTOM, DEFAULT_TOP, ids};
use crate::app::settings::Settings;

/// Two lines of fields and the weight of each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    pub top: &'static [BarField],
    pub bottom: &'static [BarField],
    pub top_bold: bool,
    pub bottom_bold: bool,
}

/// The presets in the order Settings lists them: Default, Date and time, Minimal, All.
pub static PRESETS: [Preset; 4] = [
    Preset {
        top: DEFAULT_TOP,
        bottom: DEFAULT_BOTTOM,
        top_bold: false,
        bottom_bold: true,
    },
    Preset {
        top: &[BarField::Weekday, BarField::Time, BarField::ZoneTime],
        bottom: &[BarField::BsDate, BarField::AdShort],
        top_bold: false,
        bottom_bold: true,
    },
    // One line: the bottom one is empty, so the bar shows only the bold top line.
    Preset {
        top: &[BarField::Weekday, BarField::BsDate],
        bottom: &[],
        top_bold: true,
        bottom_bold: false,
    },
    // Every field once, so the preview shows what each one looks like.
    Preset {
        top: &[
            BarField::Weekday,
            BarField::WeekdayLong,
            BarField::Day,
            BarField::MonthLong,
            BarField::MonthShort,
            BarField::Year,
            BarField::BsDate,
        ],
        bottom: &[
            BarField::BsShort,
            BarField::AdDate,
            BarField::AdShort,
            BarField::Time,
            BarField::ZoneTime,
            BarField::Tithi,
            BarField::Holiday,
        ],
        top_bold: false,
        bottom_bold: true,
    },
];

/// The preset a Settings pill stands for, or `None` for an index outside the list.
pub fn by_index(index: i32) -> Option<&'static Preset> {
    usize::try_from(index)
        .ok()
        .and_then(|index| PRESETS.get(index))
}

/// Sets both lines and both weights from `preset`.
pub fn apply(settings: &mut Settings, preset: &Preset) {
    settings.bar_top = ids(preset.top);
    settings.bar_bottom = ids(preset.bottom);
    settings.bar_top_bold = preset.top_bold;
    settings.bar_bottom_bold = preset.bottom_bold;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids_of(fields: &[BarField]) -> Vec<&'static str> {
        fields.iter().map(|field| field.id()).collect()
    }

    #[test]
    fn there_are_four_presets_in_the_listed_order() {
        assert_eq!(PRESETS.len(), 4);
        assert_eq!(PRESETS[0], *by_index(0).expect("Default"));
        assert!(by_index(4).is_none());
        assert!(by_index(-1).is_none());
    }

    #[test]
    fn default_is_the_default_lines_and_weights() {
        let mut settings = Settings {
            bar_top: ids(&[BarField::Tithi]),
            bar_bottom: ids(&[BarField::Holiday]),
            bar_top_bold: true,
            bar_bottom_bold: false,
            ..Settings::default()
        };
        apply(&mut settings, by_index(0).expect("Default"));
        let fresh = Settings::default();
        assert_eq!(settings.bar_top, fresh.bar_top);
        assert_eq!(settings.bar_bottom, fresh.bar_bottom);
        assert_eq!(settings.bar_top_bold, fresh.bar_top_bold);
        assert_eq!(settings.bar_bottom_bold, fresh.bar_bottom_bold);
        assert_eq!(ids_of(PRESETS[0].top), ["weekday", "ad_date"]);
        assert_eq!(ids_of(PRESETS[0].bottom), ["bs_date"]);
    }

    #[test]
    fn date_and_time_is_the_old_preset_with_the_bottom_line_bold() {
        let preset = &PRESETS[1];
        assert_eq!(ids_of(preset.top), ["weekday", "time", "zone_time"]);
        assert_eq!(ids_of(preset.bottom), ["bs_date", "ad_short"]);
        assert!(!preset.top_bold);
        assert!(preset.bottom_bold);
    }

    #[test]
    fn minimal_is_one_bold_line() {
        let preset = &PRESETS[2];
        assert_eq!(ids_of(preset.top), ["weekday", "bs_date"]);
        assert!(preset.bottom.is_empty());
        assert!(preset.top_bold);
    }

    #[test]
    fn all_shows_every_field_exactly_once_split_over_the_two_lines() {
        let preset = &PRESETS[3];
        let mut seen: Vec<&str> = ids_of(preset.top);
        seen.extend(ids_of(preset.bottom));
        assert_eq!(seen.len(), BarField::ALL.len());
        for field in BarField::ALL {
            assert_eq!(
                seen.iter().filter(|id| **id == field.id()).count(),
                1,
                "{} appears once",
                field.id()
            );
        }
        assert!(!preset.top.is_empty() && !preset.bottom.is_empty());
    }

    #[test]
    fn every_preset_passes_validation_unchanged() {
        for (index, preset) in PRESETS.iter().enumerate() {
            let mut settings = Settings::default();
            apply(&mut settings, preset);
            let before = settings.clone();
            assert!(settings.validate().is_empty(), "preset {index}");
            assert_eq!(settings, before, "preset {index} is a valid pair of lines");
        }
    }

    #[test]
    fn applying_a_preset_sets_the_lines_and_both_weights() {
        let mut settings = Settings::default();
        apply(&mut settings, &PRESETS[2]);
        assert_eq!(
            settings.bar_top,
            ids(&[BarField::Weekday, BarField::BsDate])
        );
        assert!(settings.bar_bottom.is_empty());
        assert!(settings.bar_top_bold);
        assert!(!settings.bar_bottom_bold);
    }
}
