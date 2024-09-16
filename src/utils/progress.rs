use std::fmt::Write;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

pub fn build_progress_bar(total: u64, message: String) -> ProgressBar {
    let mut padded_message = String::from(" ");
    padded_message.push_str(message.as_str());
    padded_message.push(' ');

    if message.is_empty() {
        padded_message = message;
    }

    ProgressBar::new(total)
        .with_message(padded_message)
        .with_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {msg} {percent}%|{wide_bar}| {pos}/{len} ({eta}, {per_sec_human})",
        )
        .unwrap()
        .with_key(
            "per_sec_human",
            |state: &ProgressState, w: &mut dyn Write| {
                write!(w, "{:.2}it/s", state.per_sec()).unwrap()
            },
        ),
    )
}
