use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    // DOOM arguments. Only iwad does anything
    #[arg(long)] pub iwad: Option<String>,
    #[arg(long, num_args = 1..)] pub file: Vec<String>,
    #[arg()] pub positional_iwad: Option<String>, // Fallback IWAD if given as bare positional arg
    #[arg(long, num_args = 1..=2)] pub warp: Vec<u8>,
    #[arg(long)] pub skill: Option<u8>,
    #[arg(long)] pub deathmatch: bool,
    #[arg(long)] pub respawn: bool,
    #[arg(long)] pub fast: bool,
    #[arg(long)] pub nomonsters: bool,
    #[arg(long)] pub record: Option<String>,
    #[arg(long)] pub playdemo: Option<String>,
    #[arg(long)] pub timedemo: Option<String>,
    #[arg(long, default_value_t = 640)] pub width: u32,
    #[arg(long, default_value_t = 400)] pub height: u32,
    #[arg(long)] pub fullscreen: bool,
}

/// Accept old DOOM-style single-dash “long” flags like `-iwad`, `-file`, etc.
pub fn normalize_doom_args<I, S>(iter: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut out = Vec::new();
    for raw in iter {
        let s: String = raw.into();
        // Convert "-word" to "--word", but leave "-" and "--" alone.
        if s.starts_with('-')
            && !s.starts_with("--")
            && s.len() > 2
            && s.as_bytes()[1].is_ascii_alphabetic()
        {
            out.push(format!("-{}", s));
        } else {
            out.push(s);
        }
    }
    out
}