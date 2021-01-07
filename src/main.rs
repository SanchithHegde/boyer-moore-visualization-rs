use anyhow::{Context, Result};
use boyer_moore::BoyerMoore;
use log::error;
use simplelog::{ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use std::{
    io::{self, Write},
    iter::repeat,
    str::from_utf8,
    thread, time,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Prints text and pattern with appropriate colors.
fn visualize(
    mut stdout: &mut StandardStream,
    text: &str,
    pattern: &str,
    t_off: usize,
    p_off: usize,
    matched: bool,
    sleep_time: f32,
) -> Result<()> {
    const ARROW_HEAD: &str = "\u{0025C0}";
    const ARROW_BODY: &str = "\u{01F89C}";
    let mut spec = ColorSpec::new();

    for i in (p_off..pattern.len()).rev() {
        if matched || i > p_off {
            write!(&mut stdout, "{}", &text[..t_off + i])?;
            stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "{}", &pattern[i..])?;
            stdout.reset()?;
            writeln!(&mut stdout, "{}", &text[t_off + pattern.len()..])?;

            write!(
                &mut stdout,
                "{}{}",
                repeat(" ").take(t_off).collect::<String>(),
                &pattern[..i],
            )?;
            stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Green)))?;
            writeln!(&mut stdout, "{}", &pattern[i..])?;
            stdout.reset()?;
        } else {
            write!(&mut stdout, "{}", &text[..t_off + p_off])?;
            stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Red)))?;
            write!(&mut stdout, "{}", &text[t_off + p_off..t_off + p_off + 1])?;
            stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "{}", &pattern[p_off + 1..])?;
            stdout.reset()?;
            writeln!(&mut stdout, "{}", &text[t_off + pattern.len()..])?;

            write!(
                &mut stdout,
                "{}{}",
                repeat(" ").take(t_off).collect::<String>(),
                &pattern[..p_off],
            )?;
            stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Red)))?;
            write!(&mut stdout, "{}", &pattern[p_off..p_off + 1])?;
            stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Green)))?;
            writeln!(&mut stdout, "{}", &pattern[p_off + 1..])?;
            stdout.reset()?;
        }

        write!(
            &mut stdout,
            "{}",
            repeat(" ").take(t_off + i).collect::<String>(),
        )?;
        stdout.set_color(spec.set_bold(true).set_fg(Some(Color::Rgb(127, 127, 127))))?;
        writeln!(
            &mut stdout,
            "{}{}",
            ARROW_HEAD,
            repeat(ARROW_BODY)
                .take(pattern.len() - i - 1)
                .collect::<String>(),
        )?;
        stdout.reset()?;

        thread::sleep(time::Duration::from_secs_f32(sleep_time));
        if i > p_off {
            print!("\u{1B}[3F");
            stdout.flush().with_context(|| "Failed to flush stdout")?;
        }
    }

    println!();
    Ok(())
}

/// Searches for all occurrences of `pattern` in `text`.
fn boyer_moore_search(
    pattern: &str,
    bm: BoyerMoore,
    text: &str,
    sleep_time: f32,
    stdout: &mut StandardStream,
) -> Result<(Vec<usize>, i32, i32)> {
    let mut occurrences = Vec::new();
    let mut alignments = 0;
    let mut comparisons = 0;

    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let mut i = 0;

    while i < text.len() - pattern.len() + 1 {
        let mut shift = 1;
        let mut mismatched = false;
        let mut mismatch_index = 0;
        let mut skip_bc = 0;
        let mut skip_gs = 0;
        alignments += 1;

        for j in (0..pattern.len()).rev() {
            comparisons += 1;

            if pattern[j] != text[i + j] {
                skip_bc = bm.bad_char_rule(j, text[i + j] as char)?;
                skip_gs = bm.good_suffix_rule(j)?;
                shift = *[shift, skip_bc, skip_gs].iter().max().unwrap();
                mismatched = true;
                mismatch_index = j;
                break;
            }
        }

        if !mismatched {
            occurrences.push(i);
            skip_gs = bm.match_skip();
            shift = *[shift, skip_gs].iter().max().unwrap();
        }

        visualize(
            stdout,
            from_utf8(text).unwrap(),
            from_utf8(pattern).unwrap(),
            i,
            mismatch_index,
            !mismatched,
            sleep_time,
        )?;
        println!("Comparisons: {}", comparisons);

        if i < text.len() - pattern.len() {
            if skip_bc > 0 {
                println!("Bad character shift: {}", skip_bc);
            }

            if skip_gs > 0 {
                println!("Good suffix shift: {}", skip_gs);
            }

            print!("Press Enter to continue ...\r");
            io::stdout()
                .flush()
                .with_context(|| "Failed to flush stdout")?;
            let mut _input = String::new();
            io::stdin()
                .read_line(&mut _input)
                .with_context(|| "Failed to read input from stdin")?;
            print!("\u{1B}[F\u{1B}[K");
            io::stdout()
                .flush()
                .with_context(|| "Failed to flush stdout")?;
        }

        println!();
        i += shift;
    }

    Ok((occurrences, alignments, comparisons))
}

fn main() {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().set_time_to_local(true).build(),
        TerminalMode::Mixed,
    )
    .map_err(|err| {
        println!("Failed to initialize logger: {}", err);
    })
    .unwrap();

    const ALPHABET: &str = "abcdefghijklmnoprstuvwxyz ";

    // Time (in seconds) to sleep between character comparisons during visualization
    const SLEEP_TIME: f32 = 0.25;

    let mut text = String::new();
    print!("Enter text    : ");
    stdout
        .flush()
        .map_err(|err| {
            error!("Failed to flush stdout: {}", err);
        })
        .unwrap();
    io::stdin()
        .read_line(&mut text)
        .map_err(|err| {
            error!("Failed to read input from stdin: {}", err);
        })
        .unwrap();
    let text = text.trim();

    let mut pattern = String::new();
    print!("Enter pattern : ");
    stdout
        .flush()
        .map_err(|err| {
            error!("Failed to flush stdout: {}", err);
        })
        .unwrap();
    io::stdin()
        .read_line(&mut pattern)
        .map_err(|err| {
            error!("Failed to read input from stdin: {}", err);
        })
        .unwrap();
    let pattern = pattern.trim();

    let bm = BoyerMoore::new(pattern, ALPHABET)
        .map_err(|err| error!("Failed to initialize Boyer-Moore object: {}", err))
        .unwrap();
    println!();

    match boyer_moore_search(pattern, bm, text, SLEEP_TIME, &mut stdout) {
        Ok((occurrences, alignments, comparisons)) => println!(
            "Text length: {}\nPattern length: {}\nOccurrences: {:#?}\nAlignments: {}\nComparisons: {}",
            text.len(),
            pattern.len(),
            occurrences,
            alignments,
            comparisons
        ),
        Err(err) => {
            for source in err.chain() {
                error!("{}", source);
            }
        }
    }
}
