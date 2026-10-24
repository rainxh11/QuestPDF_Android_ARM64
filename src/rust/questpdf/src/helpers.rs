//! Value catalogs transcribed from the QuestPDF 2026.9.1 source.
use crate::{Color, PageSizes, Size};
impl PageSizes {
    pub const POINTS_PER_INCH: u32 = 72;
    pub const A0: Size = Size::new(2384., 3370.);
    pub const A1: Size = Size::new(1684., 2384.);
    pub const A2: Size = Size::new(1191., 1684.);
    pub const A3: Size = Size::new(842., 1191.);
    pub const A4: Size = Size::new(595., 842.);
    pub const A5: Size = Size::new(420., 595.);
    pub const A6: Size = Size::new(298., 420.);
    pub const A7: Size = Size::new(210., 298.);
    pub const A8: Size = Size::new(147., 210.);
    pub const A9: Size = Size::new(105., 147.);
    pub const A10: Size = Size::new(74., 105.);
    pub const B0: Size = Size::new(2835., 4008.);
    pub const B1: Size = Size::new(2004., 2835.);
    pub const B2: Size = Size::new(1417., 2004.);
    pub const B3: Size = Size::new(1001., 1417.);
    pub const B4: Size = Size::new(709., 1001.);
    pub const B5: Size = Size::new(499., 709.);
    pub const B6: Size = Size::new(354., 499.);
    pub const B7: Size = Size::new(249., 354.);
    pub const B8: Size = Size::new(176., 249.);
    pub const B9: Size = Size::new(125., 176.);
    pub const B10: Size = Size::new(88., 125.);
    pub const C0: Size = Size::new(2599., 3677.);
    pub const C1: Size = Size::new(1837., 2599.);
    pub const C2: Size = Size::new(1298., 1837.);
    pub const C3: Size = Size::new(918., 1298.);
    pub const C4: Size = Size::new(649., 918.);
    pub const C5: Size = Size::new(459., 649.);
    pub const C6: Size = Size::new(323., 459.);
    pub const C7: Size = Size::new(230., 323.);
    pub const C8: Size = Size::new(162., 230.);
    pub const C9: Size = Size::new(113., 162.);
    pub const C10: Size = Size::new(79., 113.);
    pub const ENV10: Size = Size::new(297., 684.);
    pub const ENV_C4: Size = Size::new(649., 918.);
    pub const ENV_DL: Size = Size::new(312., 624.);
    pub const POSTCARD: Size = Size::new(284., 419.);
    pub const EXECUTIVE: Size = Size::new(522., 756.);
    pub const LETTER: Size = Size::new(612., 792.);
    pub const LEGAL: Size = Size::new(612., 1008.);
    pub const LEDGER: Size = Size::new(792., 1224.);
    pub const TABLOID: Size = Size::new(1224., 792.);
    pub const ARCH_A: Size = Size::new(648., 864.);
    pub const ARCH_B: Size = Size::new(864., 1296.);
    pub const ARCH_C: Size = Size::new(1296., 1728.);
    pub const ARCH_D: Size = Size::new(1728., 2592.);
    pub const ARCH_E: Size = Size::new(2592., 3456.);
    pub const ARCH_E1: Size = Size::new(2160., 3024.);
    pub const ARCH_E2: Size = Size::new(1872., 2736.);
    pub const ARCH_E3: Size = Size::new(1944., 2808.);
}
pub struct FontFeatures;
impl FontFeatures {
    pub const ACCESS_ALL_ALTERNATES: &'static str = "aalt";
    pub const ABOVE_BASE_FORMS: &'static str = "abvf";
    pub const ABOVE_BASE_MARK_POSITIONING: &'static str = "abvm";
    pub const ABOVE_BASE_SUBSTITUTIONS: &'static str = "abvs";
    pub const ALTERNATIVE_FRACTIONS: &'static str = "afrc";
    pub const AKHAND: &'static str = "akhn";
    pub const KERNING_FOR_ALTERNATE_PROPORTIONAL_WIDTHS: &'static str = "apkn";
    pub const BELOW_BASE_FORMS: &'static str = "blwf";
    pub const BELOW_BASE_MARK_POSITIONING: &'static str = "blwm";
    pub const BELOW_BASE_SUBSTITUTIONS: &'static str = "blws";
    pub const CONTEXTUAL_ALTERNATES: &'static str = "calt";
    pub const CASE_SENSITIVE_FORMS: &'static str = "case";
    pub const GLYPH_COMPOSITION_DECOMPOSITION: &'static str = "ccmp";
    pub const CONJUNCT_FORM_AFTER_RO: &'static str = "cfar";
    pub const CONTEXTUAL_HALF_WIDTH_SPACING: &'static str = "chws";
    pub const CONJUNCT_FORMS: &'static str = "cjct";
    pub const CONTEXTUAL_LIGATURES: &'static str = "clig";
    pub const CENTERED_CJK_PUNCTUATION: &'static str = "cpct";
    pub const CAPITAL_SPACING: &'static str = "cpsp";
    pub const CONTEXTUAL_SWASH: &'static str = "cswh";
    pub const CURSIVE_POSITIONING: &'static str = "curs";
    pub const PETITE_CAPITALS_FROM_CAPITALS: &'static str = "c2pc";
    pub const SMALL_CAPITALS_FROM_CAPITALS: &'static str = "c2sc";
    pub const DISTANCES: &'static str = "dist";
    pub const DISCRETIONARY_LIGATURES: &'static str = "dlig";
    pub const DENOMINATORS: &'static str = "dnom";
    pub const DOTLESS_FORMS: &'static str = "dtls";
    pub const EXPERT_FORMS: &'static str = "expt";
    pub const FINAL_GLYPH_ON_LINE_ALTERNATES: &'static str = "falt";
    pub const TERMINAL_FORMS2: &'static str = "fin2";
    pub const TERMINAL_FORMS3: &'static str = "fin3";
    pub const TERMINAL_FORMS: &'static str = "fina";
    pub const FLATTENED_ACCENT_FORMS: &'static str = "flac";
    pub const FRACTIONS: &'static str = "frac";
    pub const FULL_WIDTHS: &'static str = "fwid";
    pub const HALF_FORMS: &'static str = "half";
    pub const HALANT_FORMS: &'static str = "haln";
    pub const ALTERNATE_HALF_WIDTHS: &'static str = "halt";
    pub const HISTORICAL_FORMS: &'static str = "hist";
    pub const HORIZONTAL_KANA_ALTERNATES: &'static str = "hkna";
    pub const HISTORICAL_LIGATURES: &'static str = "hlig";
    pub const HANGUL: &'static str = "hngl";
    pub const HALF_WIDTHS: &'static str = "hwid";
    pub const INITIAL_FORMS: &'static str = "init";
    pub const ISOLATED_FORMS: &'static str = "isol";
    pub const ITALICS: &'static str = "ital";
    pub const JUSTIFICATION_ALTERNATES: &'static str = "jalt";
    pub const JIS78_FORMS: &'static str = "jp78";
    pub const JIS83_FORMS: &'static str = "jp83";
    pub const JIS90_FORMS: &'static str = "jp90";
    pub const JIS2004_FORMS: &'static str = "jp04";
    pub const KERNING: &'static str = "kern";
    pub const LEFT_BOUNDS: &'static str = "lfbd";
    pub const STANDARD_LIGATURES: &'static str = "liga";
    pub const LEADING_JAMO_FORMS: &'static str = "ljmo";
    pub const LINING_FIGURES: &'static str = "lnum";
    pub const LOCALIZED_FORMS: &'static str = "locl";
    pub const LEFT_TO_RIGHT_ALTERNATES: &'static str = "ltra";
    pub const LEFT_TO_RIGHT_MIRRORED_FORMS: &'static str = "ltrm";
    pub const MARK_POSITIONING: &'static str = "mark";
    pub const MEDIAL_FORMS2: &'static str = "med2";
    pub const MEDIAL_FORMS: &'static str = "medi";
    pub const MATHEMATICAL_GREEK: &'static str = "mgrk";
    pub const MARK_TO_MARK_POSITIONING: &'static str = "mkmk";
    pub const MARK_POSITIONING_VIA_SUBSTITUTION: &'static str = "mset";
    pub const ALTERNATE_ANNOTATION_FORMS: &'static str = "nalt";
    pub const NLC_KANJI_FORMS: &'static str = "nlck";
    pub const NUKTA_FORMS: &'static str = "nukt";
    pub const NUMERATORS: &'static str = "numr";
    pub const OLDSTYLE_FIGURES: &'static str = "onum";
    pub const OPTICAL_BOUNDS: &'static str = "opbd";
    pub const ORDINALS: &'static str = "ordn";
    pub const ORNAMENTS: &'static str = "ornm";
    pub const PROPORTIONAL_ALTERNATE_WIDTHS: &'static str = "palt";
    pub const PETITE_CAPITALS: &'static str = "pcap";
    pub const PROPORTIONAL_KANA: &'static str = "pkna";
    pub const PROPORTIONAL_FIGURES: &'static str = "pnum";
    pub const PRE_BASE_FORMS: &'static str = "pref";
    pub const PRE_BASE_SUBSTITUTIONS: &'static str = "pres";
    pub const POST_BASE_FORMS: &'static str = "pstf";
    pub const POST_BASE_SUBSTITUTIONS: &'static str = "psts";
    pub const PROPORTIONAL_WIDTHS: &'static str = "pwid";
    pub const QUARTER_WIDTHS: &'static str = "qwid";
    pub const RANDOMIZE: &'static str = "rand";
    pub const REQUIRED_CONTEXTUAL_ALTERNATES: &'static str = "rclt";
    pub const RAKAR_FORMS: &'static str = "rkrf";
    pub const REQUIRED_LIGATURES: &'static str = "rlig";
    pub const REPH_FORM: &'static str = "rphf";
    pub const RIGHT_BOUNDS: &'static str = "rtbd";
    pub const RIGHT_TO_LEFT_ALTERNATES: &'static str = "rtla";
    pub const RIGHT_TO_LEFT_MIRRORED_FORMS: &'static str = "rtlm";
    pub const RUBY_NOTATION_FORMS: &'static str = "ruby";
    pub const REQUIRED_VARIATION_ALTERNATES: &'static str = "rvrn";
    pub const STYLISTIC_ALTERNATES: &'static str = "salt";
    pub const SCIENTIFIC_INFERIORS: &'static str = "sinf";
    pub const OPTICAL_SIZE: &'static str = "size";
    pub const SMALL_CAPITALS: &'static str = "smcp";
    pub const SIMPLIFIED_FORMS: &'static str = "smpl";
    pub const MATH_SCRIPT_STYLE_ALTERNATES: &'static str = "ssty";
    pub const STRETCHING_GLYPH_DECOMPOSITION: &'static str = "stch";
    pub const SUBSCRIPT: &'static str = "subs";
    pub const SUPERSCRIPT: &'static str = "sups";
    pub const SWASH: &'static str = "swsh";
    pub const TITLING: &'static str = "titl";
    pub const TRAILING_JAMO_FORMS: &'static str = "tjmo";
    pub const TRADITIONAL_NAME_FORMS: &'static str = "tnam";
    pub const TABULAR_FIGURES: &'static str = "tnum";
    pub const TRADITIONAL_FORMS: &'static str = "trad";
    pub const THIRD_WIDTHS: &'static str = "twid";
    pub const UNICASE: &'static str = "unic";
    pub const ALTERNATE_VERTICAL_METRICS: &'static str = "valt";
    pub const KERNING_FOR_ALTERNATE_PROPORTIONAL_VERTICAL_METRICS: &'static str = "vapk";
    pub const VATTU_VARIANTS: &'static str = "vatu";
    pub const VERTICAL_CONTEXTUAL_HALF_WIDTH_SPACING: &'static str = "vchw";
    pub const VERTICAL_ALTERNATES: &'static str = "vert";
    pub const ALTERNATE_VERTICAL_HALF_METRICS: &'static str = "vhal";
    pub const VOWEL_JAMO_FORMS: &'static str = "vjmo";
    pub const VERTICAL_KANA_ALTERNATES: &'static str = "vkna";
    pub const VERTICAL_KERNING: &'static str = "vkrn";
    pub const PROPORTIONAL_ALTERNATE_VERTICAL_METRICS: &'static str = "vpal";
    pub const VERTICAL_ALTERNATES_AND_ROTATION: &'static str = "vrt2";
    pub const VERTICAL_ALTERNATES_FOR_ROTATION: &'static str = "vrtr";
    pub const SLASHED_ZERO: &'static str = "zero";
    pub const HOJO_KANJI_FORMS: &'static str = "hojo";
    pub fn character_variant(value: u8) -> String {
        assert!((1..=99).contains(&value));
        format!("cv{value:02}")
    }
    pub fn stylistic_set(value: u8) -> String {
        assert!((1..=20).contains(&value));
        format!("ss{value:02}")
    }
}
pub mod colors {
    use super::Color;
    pub const BLACK: Color = Color(0xFF000000);
    pub const WHITE: Color = Color(0xFFFFFFFF);
    pub const TRANSPARENT: Color = Color(0x00000000);
    pub mod red {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFFEBEE);
        pub const LIGHTEN4: Color = Color(0xFFFFCDD2);
        pub const LIGHTEN3: Color = Color(0xFFEF9A9A);
        pub const LIGHTEN2: Color = Color(0xFFE57373);
        pub const LIGHTEN1: Color = Color(0xFFEF5350);
        pub const MEDIUM: Color = Color(0xFFF44336);
        pub const DARKEN1: Color = Color(0xFFE53935);
        pub const DARKEN2: Color = Color(0xFFD32F2F);
        pub const DARKEN3: Color = Color(0xFFC62828);
        pub const DARKEN4: Color = Color(0xFFB71C1C);
        pub const ACCENT1: Color = Color(0xFFFF8A80);
        pub const ACCENT2: Color = Color(0xFFFF5252);
        pub const ACCENT3: Color = Color(0xFFFF1744);
        pub const ACCENT4: Color = Color(0xFFD50000);
    }
    pub mod pink {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFCE4EC);
        pub const LIGHTEN4: Color = Color(0xFFF8BBD0);
        pub const LIGHTEN3: Color = Color(0xFFF48FB1);
        pub const LIGHTEN2: Color = Color(0xFFF06292);
        pub const LIGHTEN1: Color = Color(0xFFEC407A);
        pub const MEDIUM: Color = Color(0xFFE91E63);
        pub const DARKEN1: Color = Color(0xFFD81B60);
        pub const DARKEN2: Color = Color(0xFFC2185B);
        pub const DARKEN3: Color = Color(0xFFAD1457);
        pub const DARKEN4: Color = Color(0xFF880E4F);
        pub const ACCENT1: Color = Color(0xFFFF80AB);
        pub const ACCENT2: Color = Color(0xFFFF4081);
        pub const ACCENT3: Color = Color(0xFFF50057);
        pub const ACCENT4: Color = Color(0xFFC51162);
    }
    pub mod purple {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFF3E5F5);
        pub const LIGHTEN4: Color = Color(0xFFE1BEE7);
        pub const LIGHTEN3: Color = Color(0xFFCE93D8);
        pub const LIGHTEN2: Color = Color(0xFFBA68C8);
        pub const LIGHTEN1: Color = Color(0xFFAB47BC);
        pub const MEDIUM: Color = Color(0xFF9C27B0);
        pub const DARKEN1: Color = Color(0xFF8E24AA);
        pub const DARKEN2: Color = Color(0xFF7B1FA2);
        pub const DARKEN3: Color = Color(0xFF6A1B9A);
        pub const DARKEN4: Color = Color(0xFF4A148C);
        pub const ACCENT1: Color = Color(0xFFEA80FC);
        pub const ACCENT2: Color = Color(0xFFE040FB);
        pub const ACCENT3: Color = Color(0xFFD500F9);
        pub const ACCENT4: Color = Color(0xFFAA00FF);
    }
    pub mod deep_purple {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFEDE7F6);
        pub const LIGHTEN4: Color = Color(0xFFD1C4E9);
        pub const LIGHTEN3: Color = Color(0xFFB39DDB);
        pub const LIGHTEN2: Color = Color(0xFF9575CD);
        pub const LIGHTEN1: Color = Color(0xFF7E57C2);
        pub const MEDIUM: Color = Color(0xFF673AB7);
        pub const DARKEN1: Color = Color(0xFF5E35B1);
        pub const DARKEN2: Color = Color(0xFF512DA8);
        pub const DARKEN3: Color = Color(0xFF4527A0);
        pub const DARKEN4: Color = Color(0xFF311B92);
        pub const ACCENT1: Color = Color(0xFFB388FF);
        pub const ACCENT2: Color = Color(0xFF7C4DFF);
        pub const ACCENT3: Color = Color(0xFF651FFF);
        pub const ACCENT4: Color = Color(0xFF6200EA);
    }
    pub mod indigo {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFE8EAF6);
        pub const LIGHTEN4: Color = Color(0xFFC5CAE9);
        pub const LIGHTEN3: Color = Color(0xFF9FA8DA);
        pub const LIGHTEN2: Color = Color(0xFF7986CB);
        pub const LIGHTEN1: Color = Color(0xFF5C6BC0);
        pub const MEDIUM: Color = Color(0xFF3F51B5);
        pub const DARKEN1: Color = Color(0xFF3949AB);
        pub const DARKEN2: Color = Color(0xFF303F9F);
        pub const DARKEN3: Color = Color(0xFF283593);
        pub const DARKEN4: Color = Color(0xFF1A237E);
        pub const ACCENT1: Color = Color(0xFF8C9EFF);
        pub const ACCENT2: Color = Color(0xFF536DFE);
        pub const ACCENT3: Color = Color(0xFF3D5AFE);
        pub const ACCENT4: Color = Color(0xFF304FFE);
    }
    pub mod blue {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFE3F2FD);
        pub const LIGHTEN4: Color = Color(0xFFBBDEFB);
        pub const LIGHTEN3: Color = Color(0xFF90CAF9);
        pub const LIGHTEN2: Color = Color(0xFF64B5F6);
        pub const LIGHTEN1: Color = Color(0xFF42A5F5);
        pub const MEDIUM: Color = Color(0xFF2196F3);
        pub const DARKEN1: Color = Color(0xFF1E88E5);
        pub const DARKEN2: Color = Color(0xFF1976D2);
        pub const DARKEN3: Color = Color(0xFF1565C0);
        pub const DARKEN4: Color = Color(0xFF0D47A1);
        pub const ACCENT1: Color = Color(0xFF82B1FF);
        pub const ACCENT2: Color = Color(0xFF448AFF);
        pub const ACCENT3: Color = Color(0xFF2979FF);
        pub const ACCENT4: Color = Color(0xFF2962FF);
    }
    pub mod light_blue {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFE1F5FE);
        pub const LIGHTEN4: Color = Color(0xFFB3E5FC);
        pub const LIGHTEN3: Color = Color(0xFF81D4FA);
        pub const LIGHTEN2: Color = Color(0xFF4FC3F7);
        pub const LIGHTEN1: Color = Color(0xFF29B6F6);
        pub const MEDIUM: Color = Color(0xFF03A9F4);
        pub const DARKEN1: Color = Color(0xFF039BE5);
        pub const DARKEN2: Color = Color(0xFF0288D1);
        pub const DARKEN3: Color = Color(0xFF0277BD);
        pub const DARKEN4: Color = Color(0xFF01579B);
        pub const ACCENT1: Color = Color(0xFF80D8FF);
        pub const ACCENT2: Color = Color(0xFF40C4FF);
        pub const ACCENT3: Color = Color(0xFF00B0FF);
        pub const ACCENT4: Color = Color(0xFF0091EA);
    }
    pub mod cyan {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFE0F7FA);
        pub const LIGHTEN4: Color = Color(0xFFB2EBF2);
        pub const LIGHTEN3: Color = Color(0xFF80DEEA);
        pub const LIGHTEN2: Color = Color(0xFF4DD0E1);
        pub const LIGHTEN1: Color = Color(0xFF26C6DA);
        pub const MEDIUM: Color = Color(0xFF00BCD4);
        pub const DARKEN1: Color = Color(0xFF00ACC1);
        pub const DARKEN2: Color = Color(0xFF0097A7);
        pub const DARKEN3: Color = Color(0xFF00838F);
        pub const DARKEN4: Color = Color(0xFF006064);
        pub const ACCENT1: Color = Color(0xFF84FFFF);
        pub const ACCENT2: Color = Color(0xFF18FFFF);
        pub const ACCENT3: Color = Color(0xFF00E5FF);
        pub const ACCENT4: Color = Color(0xFF00B8D4);
    }
    pub mod teal {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFE0F2F1);
        pub const LIGHTEN4: Color = Color(0xFFB2DFDB);
        pub const LIGHTEN3: Color = Color(0xFF80CBC4);
        pub const LIGHTEN2: Color = Color(0xFF4DB6AC);
        pub const LIGHTEN1: Color = Color(0xFF26A69A);
        pub const MEDIUM: Color = Color(0xFF009688);
        pub const DARKEN1: Color = Color(0xFF00897B);
        pub const DARKEN2: Color = Color(0xFF00796B);
        pub const DARKEN3: Color = Color(0xFF00695C);
        pub const DARKEN4: Color = Color(0xFF004D40);
        pub const ACCENT1: Color = Color(0xFFA7FFEB);
        pub const ACCENT2: Color = Color(0xFF64FFDA);
        pub const ACCENT3: Color = Color(0xFF1DE9B6);
        pub const ACCENT4: Color = Color(0xFF00BFA5);
    }
    pub mod green {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFE8F5E9);
        pub const LIGHTEN4: Color = Color(0xFFC8E6C9);
        pub const LIGHTEN3: Color = Color(0xFFA5D6A7);
        pub const LIGHTEN2: Color = Color(0xFF81C784);
        pub const LIGHTEN1: Color = Color(0xFF66BB6A);
        pub const MEDIUM: Color = Color(0xFF4CAF50);
        pub const DARKEN1: Color = Color(0xFF43A047);
        pub const DARKEN2: Color = Color(0xFF388E3C);
        pub const DARKEN3: Color = Color(0xFF2E7D32);
        pub const DARKEN4: Color = Color(0xFF1B5E20);
        pub const ACCENT1: Color = Color(0xFFB9F6CA);
        pub const ACCENT2: Color = Color(0xFF69F0AE);
        pub const ACCENT3: Color = Color(0xFF00E676);
        pub const ACCENT4: Color = Color(0xFF00C853);
    }
    pub mod light_green {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFF1F8E9);
        pub const LIGHTEN4: Color = Color(0xFFDCEDC8);
        pub const LIGHTEN3: Color = Color(0xFFC5E1A5);
        pub const LIGHTEN2: Color = Color(0xFFAED581);
        pub const LIGHTEN1: Color = Color(0xFF9CCC65);
        pub const MEDIUM: Color = Color(0xFF8BC34A);
        pub const DARKEN1: Color = Color(0xFF7CB342);
        pub const DARKEN2: Color = Color(0xFF689F38);
        pub const DARKEN3: Color = Color(0xFF558B2F);
        pub const DARKEN4: Color = Color(0xFF33691E);
        pub const ACCENT1: Color = Color(0xFFCCFF90);
        pub const ACCENT2: Color = Color(0xFFB2FF59);
        pub const ACCENT3: Color = Color(0xFF76FF03);
        pub const ACCENT4: Color = Color(0xFF64DD17);
    }
    pub mod lime {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFF9FBE7);
        pub const LIGHTEN4: Color = Color(0xFFF0F4C3);
        pub const LIGHTEN3: Color = Color(0xFFE6EE9C);
        pub const LIGHTEN2: Color = Color(0xFFDCE775);
        pub const LIGHTEN1: Color = Color(0xFFD4E157);
        pub const MEDIUM: Color = Color(0xFFCDDC39);
        pub const DARKEN1: Color = Color(0xFFC0CA33);
        pub const DARKEN2: Color = Color(0xFFAFB42B);
        pub const DARKEN3: Color = Color(0xFF9E9D24);
        pub const DARKEN4: Color = Color(0xFF827717);
        pub const ACCENT1: Color = Color(0xFFF4FF81);
        pub const ACCENT2: Color = Color(0xFFEEFF41);
        pub const ACCENT3: Color = Color(0xFFC6FF00);
        pub const ACCENT4: Color = Color(0xFFAEEA00);
    }
    pub mod yellow {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFFFDE7);
        pub const LIGHTEN4: Color = Color(0xFFFFF9C4);
        pub const LIGHTEN3: Color = Color(0xFFFFF59D);
        pub const LIGHTEN2: Color = Color(0xFFFFF176);
        pub const LIGHTEN1: Color = Color(0xFFFFEE58);
        pub const MEDIUM: Color = Color(0xFFFFEB3B);
        pub const DARKEN1: Color = Color(0xFFFDD835);
        pub const DARKEN2: Color = Color(0xFFFBC02D);
        pub const DARKEN3: Color = Color(0xFFF9A825);
        pub const DARKEN4: Color = Color(0xFFF57F17);
        pub const ACCENT1: Color = Color(0xFFFFFF8D);
        pub const ACCENT2: Color = Color(0xFFFFFF00);
        pub const ACCENT3: Color = Color(0xFFFFEA00);
        pub const ACCENT4: Color = Color(0xFFFFD600);
    }
    pub mod amber {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFFF8E1);
        pub const LIGHTEN4: Color = Color(0xFFFFECB3);
        pub const LIGHTEN3: Color = Color(0xFFFFE082);
        pub const LIGHTEN2: Color = Color(0xFFFFD54F);
        pub const LIGHTEN1: Color = Color(0xFFFFCA28);
        pub const MEDIUM: Color = Color(0xFFFFC107);
        pub const DARKEN1: Color = Color(0xFFFFB300);
        pub const DARKEN2: Color = Color(0xFFFFA000);
        pub const DARKEN3: Color = Color(0xFFFF8F00);
        pub const DARKEN4: Color = Color(0xFFFF6F00);
        pub const ACCENT1: Color = Color(0xFFFFE57F);
        pub const ACCENT2: Color = Color(0xFFFFD740);
        pub const ACCENT3: Color = Color(0xFFFFC400);
        pub const ACCENT4: Color = Color(0xFFFFAB00);
    }
    pub mod orange {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFFF3E0);
        pub const LIGHTEN4: Color = Color(0xFFFFE0B2);
        pub const LIGHTEN3: Color = Color(0xFFFFCC80);
        pub const LIGHTEN2: Color = Color(0xFFFFB74D);
        pub const LIGHTEN1: Color = Color(0xFFFFA726);
        pub const MEDIUM: Color = Color(0xFFFF9800);
        pub const DARKEN1: Color = Color(0xFFFB8C00);
        pub const DARKEN2: Color = Color(0xFFF57C00);
        pub const DARKEN3: Color = Color(0xFFEF6C00);
        pub const DARKEN4: Color = Color(0xFFE65100);
        pub const ACCENT1: Color = Color(0xFFFFD180);
        pub const ACCENT2: Color = Color(0xFFFFAB40);
        pub const ACCENT3: Color = Color(0xFFFF9100);
        pub const ACCENT4: Color = Color(0xFFFF6D00);
    }
    pub mod deep_orange {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFBE9E7);
        pub const LIGHTEN4: Color = Color(0xFFFFCCBC);
        pub const LIGHTEN3: Color = Color(0xFFFFAB91);
        pub const LIGHTEN2: Color = Color(0xFFFF8A65);
        pub const LIGHTEN1: Color = Color(0xFFFF7043);
        pub const MEDIUM: Color = Color(0xFFFF5722);
        pub const DARKEN1: Color = Color(0xFFF4511E);
        pub const DARKEN2: Color = Color(0xFFE64A19);
        pub const DARKEN3: Color = Color(0xFFD84315);
        pub const DARKEN4: Color = Color(0xFFBF360C);
        pub const ACCENT1: Color = Color(0xFFFF9E80);
        pub const ACCENT2: Color = Color(0xFFFF6E40);
        pub const ACCENT3: Color = Color(0xFFFF3D00);
        pub const ACCENT4: Color = Color(0xFFDD2C00);
    }
    pub mod brown {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFEFEBE9);
        pub const LIGHTEN4: Color = Color(0xFFD7CCC8);
        pub const LIGHTEN3: Color = Color(0xFFBCAAA4);
        pub const LIGHTEN2: Color = Color(0xFFA1887F);
        pub const LIGHTEN1: Color = Color(0xFF8D6E63);
        pub const MEDIUM: Color = Color(0xFF795548);
        pub const DARKEN1: Color = Color(0xFF6D4C41);
        pub const DARKEN2: Color = Color(0xFF5D4037);
        pub const DARKEN3: Color = Color(0xFF4E342E);
        pub const DARKEN4: Color = Color(0xFF3E2723);
    }
    pub mod grey {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFFAFAFA);
        pub const LIGHTEN4: Color = Color(0xFFF5F5F5);
        pub const LIGHTEN3: Color = Color(0xFFEEEEEE);
        pub const LIGHTEN2: Color = Color(0xFFE0E0E0);
        pub const LIGHTEN1: Color = Color(0xFFBDBDBD);
        pub const MEDIUM: Color = Color(0xFF9E9E9E);
        pub const DARKEN1: Color = Color(0xFF757575);
        pub const DARKEN2: Color = Color(0xFF616161);
        pub const DARKEN3: Color = Color(0xFF424242);
        pub const DARKEN4: Color = Color(0xFF212121);
    }
    pub mod blue_grey {
        use super::Color;
        pub const LIGHTEN5: Color = Color(0xFFECEFF1);
        pub const LIGHTEN4: Color = Color(0xFFCFD8DC);
        pub const LIGHTEN3: Color = Color(0xFFB0BEC5);
        pub const LIGHTEN2: Color = Color(0xFF90A4AE);
        pub const LIGHTEN1: Color = Color(0xFF78909C);
        pub const MEDIUM: Color = Color(0xFF607D8B);
        pub const DARKEN1: Color = Color(0xFF546E7A);
        pub const DARKEN2: Color = Color(0xFF455A64);
        pub const DARKEN3: Color = Color(0xFF37474F);
        pub const DARKEN4: Color = Color(0xFF263238);
    }
}
