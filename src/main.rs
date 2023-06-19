mod symbol;

use std::fmt::{self, Display};

use clap::{Parser, ValueEnum};
use pwhash::sha512_crypt::hash;

use anyhow::{bail, Result};
use symbol::Symbols;

const DEFAULT_MAX_LEN: &str = "72";

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
    let args = Args::parse();
    if let Err(e) = args.run() {
        eprintln!("Error: {}", e);
    }
}

#[derive(Debug, Parser)]
struct Args {
    /// Generator mode
    #[clap(short = 'm', long, default_value = "chars")]
    mode: Mode,

    /// Use lower cases. (chars mode only)
    #[clap(short = 'a', long)]
    lower: bool,
    /// Use catital cases. (chars mode only)
    #[clap(short = 'A', long)]
    capital: bool,
    /// Use digits. (chars mode only)
    #[clap(short = '0', long)]
    digit: bool,
    /// Use all ASCII symbols (except ' '). (chars mode only)
    #[clap(short = '!', long)]
    all_symbols: bool,
    /// Use specified symbols. (chars mode only)
    #[clap(short = 's', long)]
    symbols: Option<String>,
    /// Allow non-appear symbols. (chars mode only)
    #[clap(long)]
    non_appear: bool,

    /// Words separator. (words mode only)
    #[clap(short = 'S', long, default_value = " ")]
    sep: String,

    /// Length of symbols sequence.
    #[clap(short = 'L', long)]
    length: Option<usize>,
    /// Entropy requirement.
    #[clap(short = 'E', long, default_value = "71.45")]
    entropy: f64,
    /// Maximum length of symbols sequence.
    #[clap(short = 'M', long, default_value = DEFAULT_MAX_LEN)]
    max_len: usize,

    /// Print UNIX crypt() hash.
    #[clap(short = 'H', long)]
    hash: bool,

    /// Print different N passwords.
    #[clap(short = 'N', long, default_value = "1")]
    count: usize,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    Chars,
    BasicWords,
    Diceware,
    DicewareAlnum,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Chars => write!(f, "chars"),
            Mode::BasicWords => write!(f, "basic-words"),
            Mode::Diceware => write!(f, "diceware"),
            Mode::DicewareAlnum => write!(f, "diceware-alnum"),
        }
    }
}

impl Args {
    fn run(&self) -> Result<()> {
        warn_entropy(self.entropy);

        for _ in 0..self.count {
            let password = match self.mode {
                Mode::Chars => self.char_mode()?,
                _ => self.words_mode()?,
            };

            print!("{}", password);
            if self.hash {
                let hash = hash(password.as_bytes())?;
                print!("\t{}", hash);
            }
            println!();
        }

        Ok(())
    }

    fn char_mode(&self) -> Result<String> {
        let mut chars = String::new();
        let mut groups = Vec::<&str>::new();
        if self.lower {
            let s = "abcdefghijklmnopqrstuvwxyz";
            groups.push(s);
            chars.push_str(s);
        }
        if self.capital {
            let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            groups.push(s);
            chars.push_str(s);
        }
        if self.digit {
            let s = "0123456789";
            groups.push(s);
            chars.push_str(s);
        }
        if self.all_symbols {
            let s = "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";
            groups.push(s);
            chars.push_str(s);
        } else if let Some(ref s) = self.symbols {
            groups.push(s);
            chars.push_str(s);
        }

        if chars.is_empty() {
            bail!("No characters.");
        }

        let symbols = Symbols::from_chars(chars.chars());

        let validate =
            |s: &str| self.non_appear || groups.iter().all(|g| s.chars().any(|c| g.contains(c)));

        self.generate(symbols, "", validate)
    }

    fn words_mode(&self) -> Result<String> {
        let words = match self.mode {
            Mode::BasicWords => &include_bytes!("../resources/basic-words.txt")[..],
            Mode::Diceware => &include_bytes!("../resources/diceware.txt")[..],
            Mode::DicewareAlnum => &include_bytes!("../resources/diceware-alnum.txt")[..],
            m => panic!("Invalid mode: {}", m),
        };
        let symbols = Symbols::from_bufread(words)?;

        let sep = self.sep.as_str();

        self.generate(symbols, sep, |s| s.len() <= self.max_len)
    }

    fn generate(
        &self,
        symbols: Symbols,
        sep: &str,
        validate: impl Fn(&str) -> bool,
    ) -> Result<String> {
        match self.length {
            Some(length) => {
                let ee = symbols.estimate_entropy(length, sep, &validate)?;
                if ee == 0.0 {
                    bail!("It is impossible to meet the conditions.");
                }

                let password = symbols.generate(length, sep, &validate);
                Ok(password)
            }
            None => {
                let base = symbols.base_entropy(1);
                let minimum_len = (self.entropy / base).ceil() as usize;

                for length in minimum_len.. {
                    let ee = symbols.estimate_entropy(length, sep, &validate)?;

                    if ee == 0.0 {
                        bail!("Never met entropy requirement.");
                    }

                    if ee >= self.entropy {
                        let password = symbols.generate(length, sep, &validate);
                        return Ok(password);
                    }
                }
                unreachable!()
            }
        }
    }
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
