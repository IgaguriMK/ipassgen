mod err;
mod symbol;

use clap::{crate_authors, crate_description, crate_name, App, Arg, ArgMatches};

use err::Error;
use symbol::Symbols;

const DEFAULT_CHARS_LEN: usize = 12;
const DEFAULT_WORDS_LEN: usize = 6;
const DEFAULT_MAX_LEN: usize = std::usize::MAX;

// 2016年に、Nvidia GTX 1080を8台の構成でベンチマークとして以下のハッシュレートが達成されている。
// https://gist.github.com/epixoip/a83d38f412b4737e99bbef804a270c40
//
// crypt(MD5):     79711.6 kH/s
// crypt(SHA256):   3110.0 kH/s
// crypt(SHA512):   1168.6 kH/s
// bcrypt:           105.7 kH/s
// scrypt:          3493.6 kH/s
//
// RTX 2080 Tiでは倍程度の速度が出るようである。
// https://gist.github.com/binary1985/c8153c8ec44595fdabbf03157562763e
//
// 個人でも用意可能な環境としてRTX 2080 Tiの2台構成を想定し、MD5は非推奨であるため、crypt(SHA-256)で1900 kH/sを基準とする。
// これは 秒間20.86 bit分である。
//
// 24時間 => 37.26 bit
// 1週間  => 40.06 bit
//
// この値にマージンとして約5bit追加し、45 bit未満を非常に危険（ENTROPY_CRITICAL_WARN）とした。
const ENTROPY_CRITICAL_WARN: f64 = 45.0;

// ASCIIの空白を除く全記号を使った英数字記号8文字が52.44 bitであり新規では非推奨なので、それを少し超える55 bit未満を警告（ENTROPY_WARN）とした。
const ENTROPY_WARN: f64 = 55.0;

fn main() {
    if let Err(e) = w_main() {
        eprintln!("Error: {}", e);
    }
}

fn w_main() -> Result<(), Error> {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .possible_values(&[
                    "chars",
                    "words-basic",
                    "words-diceware",
                    "words-diceware-alnum",
                ])
                .default_value("chars")
                .help("Generator mode."),
        )
        .arg(
            Arg::with_name("lower")
                .short("a")
                .long("lower")
                .help("Use lower cases. (chars mode only)"),
        )
        .arg(
            Arg::with_name("captial")
                .short("A")
                .long("cap")
                .help("Use catital cases. (chars mode only)"),
        )
        .arg(
            Arg::with_name("digit")
                .short("0")
                .long("digit")
                .help("Use digits. (chars mode only)"),
        )
        .arg(
            Arg::with_name("all_symbols")
                .short("!")
                .long("all-symbols")
                .help("Use all ASCII symbols (except ' '). (chars mode only)"),
        )
        .arg(
            Arg::with_name("symbols")
                .short("s")
                .long("symbols")
                .takes_value(true)
                .help("Use specified symbols. (chars mode only)"),
        )
        .arg(
            Arg::with_name("sep")
                .short("S")
                .long("sep")
                .default_value(" ")
                .help("Words separator. (words mode only)"),
        )
        .arg(
            Arg::with_name("length")
                .short("L")
                .long("length")
                .takes_value(true)
                .help("Length of symbols sequence."),
        )
        .arg(
            Arg::with_name("entropy")
                .short("E")
                .long("entropy")
                .takes_value(true)
                .help("Entropy requirement."),
        )
        .arg(
            Arg::with_name("max_length")
                .short("M")
                .long("max-length")
                .takes_value(true)
                .help("Maximum length in byte."),
        )
        .get_matches();

    match matches.value_of("mode").unwrap() {
        "chars" => char_mode(&matches),
        mode => words_mode(&matches, mode),
    }
}

fn char_mode(matches: &ArgMatches) -> Result<(), Error> {
    let mut chars = String::new();
    if matches.is_present("lower") {
        chars.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if matches.is_present("captial") {
        chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if matches.is_present("digit") {
        chars.push_str("0123456789");
    }
    if matches.is_present("all_symbols") {
        chars.push_str("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~");
    } else if let Some(ss) = matches.value_of("symbols") {
        chars.push_str(ss);
    }

    if chars.is_empty() {
        return Err(Error::new("No characters.".to_owned()));
    }

    let symbols = Symbols::from_chars(chars.chars());
    generate(matches, symbols, "", DEFAULT_CHARS_LEN)
}

fn words_mode(matches: &ArgMatches, mode: &str) -> Result<(), Error> {
    let words = match mode {
        "words-basic" => &include_bytes!("../resources/basic-words.txt")[..],
        "words-diceware" => &include_bytes!("../resources/diceware.txt")[..],
        "words-diceware-alnum" => &include_bytes!("../resources/diceware-alnum.txt")[..],
        m => panic!("Invalid mode: {}", m),
    };
    let symbols = Symbols::from_bufread(words)?;

    let sep = matches.value_of("sep").unwrap();

    generate(matches, symbols, sep, DEFAULT_WORDS_LEN)
}

fn generate(
    matches: &ArgMatches,
    symbols: Symbols,
    sep: &str,
    default_len: usize,
) -> Result<(), Error> {
    let length = get_usize(&matches, "length")?;
    let entropy = get_f64(&matches, "entropy")?;
    let max_len = get_usize(&matches, "max_length")?.unwrap_or(DEFAULT_MAX_LEN);

    match (length, entropy) {
        (Some(length), Some(entropy)) => {
            let ee = symbols.estimate_entropy(length, sep.len(), max_len)?;
            if ee < entropy {
                return Err(Error::new(format!(
                    "Required entropy is {}, but only {:.2}",
                    entropy, ee
                )));
            }
            let password = symbols.generate(length, sep, max_len)?;
            println!("{}", password);
        }
        (None, Some(entropy)) => {
            for length in 4.. {
                let ee = symbols.estimate_entropy(length, sep.len(), max_len)?;

                if ee == 0.0 {
                    return Err(Error::new("Never met entropy requirement.".to_owned()));
                }

                if ee >= entropy {
                    let password = symbols.generate(length, sep, max_len)?;
                    println!("{}", password);
                    break;
                }
            }
        }
        (Some(length), None) => {
            warn_entropy(symbols.estimate_entropy(length, sep.len(), max_len)?);
            let password = symbols.generate(length, sep, max_len)?;
            println!("{}", password);
        }
        (None, None) => {
            warn_entropy(symbols.estimate_entropy(default_len, sep.len(), max_len)?);
            let password = symbols.generate(default_len, sep, max_len)?;
            println!("{}", password);
        }
    }

    Ok(())
}

fn warn_entropy(ee: f64) {
    if ee < ENTROPY_CRITICAL_WARN {
        eprintln!("CRITICAL WARNING: This setting is too weak ({:.2} bits < {} bits). May be cracked by personal attackers.", ee, ENTROPY_CRITICAL_WARN);
        return;
    }

    if ee < ENTROPY_WARN {
        eprintln!(
            "WARNING: This setting is weak ({:.2} bits < {} bits).",
            ee, ENTROPY_WARN
        );
        return;
    }
}

fn get_usize(matches: &ArgMatches, key: &str) -> Result<Option<usize>, Error> {
    if let Some(s) = matches.value_of(key) {
        Ok(Some(s.parse()?))
    } else {
        Ok(None)
    }
}

fn get_f64(matches: &ArgMatches, key: &str) -> Result<Option<f64>, Error> {
    if let Some(s) = matches.value_of(key) {
        Ok(Some(s.parse()?))
    } else {
        Ok(None)
    }
}
