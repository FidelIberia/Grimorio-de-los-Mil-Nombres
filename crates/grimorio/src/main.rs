//! CLI for Grimorio de los Mil Nombres — optional uplink params, defaults filled internally.

use std::env;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use grimorio::{answer, Language, UplinkDraft, VoiceKind};

const MAX_COUNT: u32 = 200;

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return ExitCode::SUCCESS;
    }

    let cmd = args.remove(0);
    match cmd.as_str() {
        "help" | "-h" | "--help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "answer" => match run_answer(&args, None) {
            Ok(()) => ExitCode::SUCCESS,
            Err(msg) => fail(&msg),
        },
        "epithet" | "epithets" => match run_answer(&args, Some(VoiceKind::SampleEpithet)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(msg) => fail(&msg),
        },
        "wisdom" => match run_answer(&args, Some(VoiceKind::Wisdom)) {
            Ok(()) => ExitCode::SUCCESS,
            Err(msg) => fail(&msg),
        },
        other => fail(&format!("unknown command `{other}`")),
    }
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("error: {msg}");
    eprintln!("Try `grimorio help`.");
    ExitCode::from(2)
}

/// Shared runner: optional uplink fields + CLI-only `--count`.
fn run_answer(args: &[String], force_voice: Option<VoiceKind>) -> Result<(), String> {
    let mut draft = UplinkDraft::default();
    let mut count: u32 = 1;
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "help" | "-h" | "--help" => {
                print_answer_help();
                return Ok(());
            }
            "--lang" | "-l" | "--language" => {
                draft.language = Some(parse_language(need_value(args, i, arg)?)?);
                i += 2;
            }
            flag if flag.starts_with("--lang=") => {
                draft.language = Some(parse_language(&flag[7..])?);
                i += 1;
            }
            "--voice" | "-v" | "--kind" => {
                draft.voice = Some(parse_voice(need_value(args, i, arg)?)?);
                i += 2;
            }
            flag if flag.starts_with("--voice=") => {
                draft.voice = Some(parse_voice(&flag[8..])?);
                i += 1;
            }
            "--english" | "--en" => {
                draft.language = Some(Language::English);
                i += 1;
            }
            "--spanish" | "--es" => {
                draft.language = Some(Language::Spanish);
                i += 1;
            }
            "--russian" | "--rusian" | "--ru" => {
                draft.language = Some(Language::Russian);
                i += 1;
            }
            // CLI-only: how many downlinks to print (bumps sample/generation index).
            "--count" | "-n" | "-c" => {
                count = parse_count(need_value(args, i, arg)?)?;
                i += 2;
            }
            flag if flag.starts_with("--count=") => {
                count = parse_count(&flag[8..])?;
                i += 1;
            }
            // Sample / shared seed
            "--seed" | "-s" => {
                draft.seed = Some(parse_u64(need_value(args, i, arg)?, "--seed")?);
                i += 2;
            }
            flag if flag.starts_with("--seed=") => {
                draft.seed = Some(parse_u64(&flag[7..], "--seed")?);
                i += 1;
            }
            "--index" | "-i" => {
                draft.index = Some(parse_u32(need_value(args, i, arg)?, "--index")?);
                i += 2;
            }
            flag if flag.starts_with("--index=") => {
                draft.index = Some(parse_u32(&flag[8..], "--index")?);
                i += 1;
            }
            // Wisdom
            "--mix" => {
                draft.mix = Some(parse_u64(need_value(args, i, arg)?, "--mix")?);
                i += 2;
            }
            flag if flag.starts_with("--mix=") => {
                draft.mix = Some(parse_u64(&flag[6..], "--mix")?);
                i += 1;
            }
            "--tone" | "--dominant-tone" => {
                draft.dominant_tone_idx = Some(parse_u8(need_value(args, i, arg)?, "--tone")?);
                i += 2;
            }
            "--scar-mass" => {
                draft.scar_mass = Some(parse_u32(need_value(args, i, arg)?, "--scar-mass")?);
                i += 2;
            }
            "--rare-event" => {
                draft.rare_event_token =
                    Some(parse_u64(need_value(args, i, arg)?, "--rare-event")?);
                i += 2;
            }
            "--dying-viability" => {
                draft.dying_viability_quant =
                    Some(parse_u8(need_value(args, i, arg)?, "--dying-viability")?);
                i += 2;
            }
            "--fossil-absence" => {
                draft.fossil_absence_mass =
                    Some(parse_u32(need_value(args, i, arg)?, "--fossil-absence")?);
                i += 2;
            }
            "--attention" => {
                draft.attention_xor = Some(parse_u64(need_value(args, i, arg)?, "--attention")?);
                i += 2;
            }
            "--soul" => {
                draft.soul_attunement_quant =
                    Some(parse_u8(need_value(args, i, arg)?, "--soul")?);
                i += 2;
            }
            // Generation epithet / standout
            "--generation" | "-g" => {
                draft.generation = Some(parse_u32(need_value(args, i, arg)?, "--generation")?);
                i += 2;
            }
            flag if flag.starts_with("--generation=") => {
                draft.generation = Some(parse_u32(&flag[13..], "--generation")?);
                i += 1;
            }
            "--label" => {
                draft.label = Some(need_value(args, i, arg)?.to_string());
                i += 2;
            }
            flag if flag.starts_with("--label=") => {
                draft.label = Some(flag[8..].to_string());
                i += 1;
            }
            "--fitness" => {
                draft.fitness = Some(parse_f32(need_value(args, i, arg)?, "--fitness")?);
                i += 2;
            }
            "--viability" => {
                draft.viability = Some(parse_f32(need_value(args, i, arg)?, "--viability")?);
                i += 2;
            }
            "--vitality" => {
                draft.vitality = Some(parse_f32(need_value(args, i, arg)?, "--vitality")?);
                i += 2;
            }
            "--resonance" => {
                draft.resonance = Some(parse_f32(need_value(args, i, arg)?, "--resonance")?);
                i += 2;
            }
            "--mutation" | "--mutation-pressure" => {
                draft.mutation_pressure =
                    Some(parse_f32(need_value(args, i, arg)?, "--mutation")?);
                i += 2;
            }
            "--symbolic" | "--symbolic-density" => {
                draft.symbolic_density =
                    Some(parse_f32(need_value(args, i, arg)?, "--symbolic")?);
                i += 2;
            }
            "--memory" | "--memory-depth" => {
                draft.memory_depth = Some(parse_f32(need_value(args, i, arg)?, "--memory")?);
                i += 2;
            }
            "--shadow" | "--shadow-pull" => {
                draft.shadow_pull = Some(parse_f32(need_value(args, i, arg)?, "--shadow")?);
                i += 2;
            }
            "--myth" => {
                draft.myth = Some(need_value(args, i, arg)?.to_string());
                i += 2;
            }
            flag if flag.starts_with("--myth=") => {
                draft.myth = Some(flag[7..].to_string());
                i += 1;
            }
            other => return Err(format!("unknown option `{other}`")),
        }
    }

    if draft.voice.is_none() {
        if let Some(v) = force_voice {
            // `epithet` defaults to sample, but standout hints promote to generation_epithet.
            if v == VoiceKind::SampleEpithet && draft.has_standout_hints() {
                draft.voice = Some(VoiceKind::GenerationEpithet);
            } else {
                draft.voice = Some(v);
            }
        }
    }

    if draft.seed.is_none() {
        draft.seed = Some(entropy_seed());
    }

    let start_index = draft.index.unwrap_or(0);
    for offset in 0..count {
        let mut one = draft.clone();
        one.index = Some(start_index.saturating_add(offset));
        // For wisdom batches, bump mix so lines can differ.
        if one.resolve_voice() == VoiceKind::Wisdom {
            let base_mix = one.mix.or(one.seed).unwrap_or(0);
            one.mix = Some(base_mix.wrapping_add(offset as u64));
        }
        let down = answer(&one.finish());
        if !down.empty {
            println!("{}", down.text);
        }
    }
    Ok(())
}

fn need_value<'a>(args: &'a [String], i: usize, flag: &str) -> Result<&'a str, String> {
    args.get(i + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn parse_language(raw: &str) -> Result<Language, String> {
    Language::from_cli_flag(raw.trim()).ok_or_else(|| {
        format!("invalid language `{raw}` (use en|es|ru / english|spanish|russian)")
    })
}

fn parse_voice(raw: &str) -> Result<VoiceKind, String> {
    VoiceKind::parse(raw).ok_or_else(|| {
        format!(
            "invalid voice `{raw}` (use wisdom|generation_epithet|sample_epithet)"
        )
    })
}

fn parse_count(raw: &str) -> Result<u32, String> {
    let n = parse_u32(raw, "--count")?;
    if n == 0 || n > MAX_COUNT {
        return Err(format!("--count must be 1..={MAX_COUNT}"));
    }
    Ok(n)
}

fn parse_u64(raw: &str, flag: &str) -> Result<u64, String> {
    raw.trim()
        .parse::<u64>()
        .map_err(|_| format!("invalid {flag} value `{raw}` (expected unsigned integer)"))
}

fn parse_u32(raw: &str, flag: &str) -> Result<u32, String> {
    raw.trim()
        .parse::<u32>()
        .map_err(|_| format!("invalid {flag} value `{raw}` (expected unsigned integer)"))
}

fn parse_u8(raw: &str, flag: &str) -> Result<u8, String> {
    raw.trim()
        .parse::<u8>()
        .map_err(|_| format!("invalid {flag} value `{raw}` (expected 0..=255)"))
}

fn parse_f32(raw: &str, flag: &str) -> Result<f32, String> {
    raw.trim()
        .parse::<f32>()
        .map_err(|_| format!("invalid {flag} value `{raw}` (expected number)"))
}

fn entropy_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0xC0FFEE)
}

fn print_help() {
    println!(
        "\
grimorio — Grimorio de los Mil Nombres

USAGE:
    grimorio <command> [uplink options]

COMMANDS:
    help                 Show this help
    answer [options]     Build uplink (partial ok) → downlink
    epithet [options]    Same as answer; default voice=sample_epithet
    wisdom [options]     Same as answer; default voice=wisdom

Uplink fields are optional. Omitted fields are filled internally.
Standout hints (e.g. --label, --shadow) select generation_epithet when voice is unset.
Wisdom hints (e.g. --mix, --scar-mass) select wisdom when voice is unset.

EXAMPLES:
    grimorio answer
    grimorio answer --voice wisdom --mix 42 --scar-mass 10
    grimorio answer --label keeper --shadow 0.9 --lang ru
    grimorio epithet --count 10 --seed 1
    grimorio wisdom --tone 3 --soul 200
    grimorio answer --help
"
    );
}

fn print_answer_help() {
    println!(
        "\
grimorio answer — uplink params (all optional) → downlink text

USAGE:
    grimorio answer|epithet|wisdom [options]

COMMON:
    -h, --help
    -l, --lang <en|es|ru>
        --english / --spanish / --russian
    -v, --voice <wisdom|generation_epithet|sample_epithet>
    -n, --count <N>          Print N lines (CLI only; max {MAX_COUNT})
    -s, --seed <N>           Fill seed / default mix / sample base
    -i, --index <N>          Sample index / portrait sample base

WISDOM uplink:
    --mix <N>
    --tone <0..255>          dominant_tone_idx
    --scar-mass <N>
    --rare-event <N>
    --dying-viability <0..255>
    --fossil-absence <N>
    --attention <N>
    --soul <0..255>

GENERATION EPITHET uplink (standout):
    -g, --generation <N>
    --label <TEXT>
    --fitness <F>
    --viability <F>
    --vitality <F>
    --resonance <F>
    --mutation <F>
    --symbolic <F>
    --memory <F>
    --shadow <F>
    --myth <TEXT>

SAMPLE EPITHET uplink:
    Uses --index and --seed (defaults: 0 and entropy).

NOTES:
    Partial uplinks are completed by UplinkDraft::finish.
    `epithet` + standout flags → generation_epithet automatically.

EXAMPLES:
    grimorio answer --shadow 0.8 --resonance 0.6
    grimorio answer --voice wisdom --mix 7
    grimorio epithet -n 5 --seed 424242 --spanish
    grimorio wisdom --scar-mass 40 --count 3
"
    );
}
