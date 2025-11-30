use regex::Regex;

use crate::{
    ffi::libqalculate::ffi,
    widgets::windows::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule}
};

/// Table of number suffixes & amount of zeroes needed to reach them
const NUM_SUFFIXES: [(u16, &str); 102] = [
    (3, "k"), (6, "M"), (9, "B"), (12, "T"),
    (15, "qd"), (18, "Qn"), (21, "sx"), (24, "Sp"),
    (27, "O"), (30, "N"), (33, "de"), (36, "Ud"),
    (39, "DD"), (42, "tdD"), (45, "qdD"), (48, "QnD"),
    (51, "sxD"), (54, "SpD"), (57, "OcD"), (60, "NvD"),
    (63, "Vgn"), (66, "UVg"), (69, "DVg"), (72, "TVg"),
    (75, "qtV"), (78, "QnV"), (81, "SeV"), (84, "SPG"), (87, "OVG"),
    (90, "NVG"), (93, "TGN"), (96, "UTG"), (99, "DTG"),
    (102, "tsTG"), (105, "qtTG"), (108, "QnTG"), (111, "ssTG"),
    (114, "SpTG"), (117, "OcTG"), (120, "NoTG"), (123, "QdDR"),
    (126, "uQDR"), (129, "dQDR"), (132, "tQDR"), (135, "qdQDR"),
    (138, "QnQDR"), (141, "sxQDR"), (144, "SpQDR"), (147, "OQDDr"),
    (150, "NQDDr"), (153, "qQGNT"), (156, "uQGNT"), (159, "dQGNT"), 
    (162, "tQGNT"), (165, "qdQGNT"), (168, "QnQGNT"), (171, "sxQGNT"),
    (174, "SpQGNT"), (177, "OQQGNT"), (180, "NQQGNT"), (183, "SXGNTL"),
    (186, "USXGNTL"), (189, "DSXGNTL"), (192, "TSXGNTL"), (195, "QTSXGNTL"),
    (198, "QNSXGNTL"), (201, "SXSXGNTL"), (204, "SPSXGNTL"), (207, "OSXGNTL"),
    (210, "NVSXGNTL"), (213, "SPTGNTL"), (216, "USPTGNTL"), (219, "DSPTGNTL"),
    (222, "TSPTGNTL"), (225, "QTSPTGNTL"), (228, "QNSPTGNTL"), (231, "SXSPTGNTL"),
    (234, "SPSPTGNTL"), (237, "OSPTGNTL"), (240, "NVSPTGNTL"), (243, "OTGNTL"),
    (246, "UOTGNTL"), (249, "DOTGNTL"), (252, "TOTGNTL"), (255, "QTOTGNTL"),
    (258, "QNOTGNTL"), (261, "SXOTGNTL"), (264, "SPOTGNTL"), (267, "OTOTGNTL"),
    (270, "NVOTGNTL"), (273, "NONGNTL"), (276, "UNONGNTL"), (279, "DNONGNTL"),
    (282, "TNONGNTL"), (285, "QTNONGNTL"), (288, "QNNONGNTL"), (291, "SXNONGNTL"),
    (294, "SPNONGNTL"), (297, "OTNONGNTL"), (300, "NONONGNTL"),
    (303, "CENT"),
    (306, "UNCENT")
];

fn to_suffixed_number(num: f64) -> String {
    if num.is_nan() {
        return "NaN".to_owned();
    }

    if num.is_infinite() {
        return "inf".to_owned();
    }

    if num.abs() < 1000.0 {
        return num.to_string();
    }

    let zeroes = (num.abs().log10().floor() as u16 / 3) * 3;
    let num = num / 1000.0_f64.powi((zeroes / 3) as i32);
    let suffix = NUM_SUFFIXES.iter()
        .find(|(z, _)| *z == zeroes)
        .map_or("", |(_, s)| *s);

    let num_str = if is_scientific_notation(&num.to_string()) {
        format!("{:.2e}", num)
    } else {
        format!("{:.2}", num)
    };

    format!("{}{}", num_str, suffix)
}

fn is_scientific_notation(num: &str) -> bool {
    let notation_regex = Regex::new(r"^\d(?:\.\d+)?[eE][-+]?\d+$").unwrap();

    notation_regex.is_match(num)
}

/// Instead of just giving a scientific notation, we can add a suffix to it
/// to make it clearer what the number is
/// e.g. "1.2e+13" -> "1.2e+13 (12T)"
fn add_suffix_to_notation(num: &str) -> String {
    if is_scientific_notation(num) {
        format!(
            "{} ({})",
            num,
            to_suffixed_number(num.parse::<f64>().unwrap_or_default())
        )
    } else {
        num.to_owned()
    }
}

pub struct OverviewCalculatorModule;

impl OverviewSearchModule for OverviewCalculatorModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["calc", "qalc", "c", "="]
    }

    fn icon(&self) -> &str {
        "calculate"
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        let unlocalized = ffi::unlocalizeExpression(query.to_owned());
        let result = add_suffix_to_notation(&ffi::calculateAndPrint(unlocalized, 1000));

        vec![OverviewSearchItem::new(
            "calculator-result".to_owned(),
            result.clone(),
            Some("Math result".to_owned()),
            "accessories-calculator".to_owned(),
            "copy".to_owned(),
            OverviewSearchItemAction::Copy(result),
            None
        )]
    }
}