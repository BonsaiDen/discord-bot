// STD Dependencies -----------------------------------------------------------
use std::cmp;


// Text Utilities -------------------------------------------------------------
pub fn list_words(
    title: &str,
    words: &[&str],
    block_size: usize,
    line_size: usize

) -> Vec<String> {

    let total = words.len();
    words.chunks(block_size).enumerate().map(|(index, block)| {

        let lines: Vec<String> = block.chunks(line_size).map(|c| {
            c.join(", ")

        }).collect();

        let offset = index * block_size + 1;
        format!(
            "\n__{} {} - {} of {}:__\n\n - {}",
            title,
            offset,
            cmp::min(offset + (block_size - 1), total),
            total,
            lines.join("\n - ")
        )

    }).collect()

}

pub fn list_lines(
    title: &str,
    lines: &[String],
    line_size: usize

) -> Vec<String> {

    let total = lines.len();
    lines.chunks(line_size).enumerate().map(|(index, lines)| {

        let offset = index * line_size + 1;
        format!(
            "\n__{} {} - {} of {}:__\n\n - {}",
            title,
            offset,
            cmp::min(offset + (line_size - 1), total),
            total,
            lines.join("\n - ")
        )

    }).collect()

}

