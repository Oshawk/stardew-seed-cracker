//! Simulates notice-board quest generation for a given seed across a range of days.
//!
//! Run with:
//!   cargo run --bin simulate -- [OPTIONS]
//!
//! Options:
//!   --seed <N>        uniqueIDForThisGame (default 0)
//!   --platform <P>    pc | switch (default pc)
//!   --day <N>         starting day of month, 1–28 (default 1)
//!   --season <S>      spring | summer | fall | winter (default spring)
//!   --year <N>        starting year, ≥1 (default 1)
//!   --count <N>       number of days to generate (default 28)

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    native::run();
}

// Everything that depends on `clap` lives here so the WASM target sees only
// the empty `main()` above and does not try to resolve the clap crate.
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use clap::Parser;

    use stardew_seed_cracker::codegen::OBJECTS;
    use stardew_seed_cracker::game_data::{resource_type_to_items, FISHING_POOLS};
    use stardew_seed_cracker::observation::{Platform, Season};
    use stardew_seed_cracker::prng::{create_day_save_random, create_initialisation_random};

    // ── CLI ───────────────────────────────────────────────────────────────────

    #[derive(Parser)]
    #[command(about = "Simulate Stardew Valley 1.6 notice-board quests for a known seed")]
    struct Args {
        /// uniqueIDForThisGame (seconds since Stardew epoch)
        #[arg(long, default_value_t = 0)]
        seed: u64,

        /// Platform: pc or switch
        #[arg(long, default_value = "pc", value_parser = parse_platform)]
        platform: Platform,

        /// Starting day of month (1–28)
        #[arg(long, default_value_t = 1)]
        day: u8,

        /// Starting season: spring, summer, fall, or winter
        #[arg(long, default_value = "spring", value_parser = parse_season)]
        season: Season,

        /// Starting year (≥1)
        #[arg(long, default_value_t = 1)]
        year: u32,

        /// Number of days to generate
        #[arg(long, default_value_t = 28)]
        count: u32,
    }

    fn parse_platform(s: &str) -> Result<Platform, String> {
        match s.to_ascii_lowercase().as_str() {
            "pc" => Ok(Platform::PC),
            "switch" => Ok(Platform::Switch),
            _ => Err(format!("unknown platform '{s}' — use 'pc' or 'switch'")),
        }
    }

    fn parse_season(s: &str) -> Result<Season, String> {
        Season::from_key(&s.to_ascii_lowercase())
            .ok_or_else(|| format!("unknown season '{s}' — use spring, summer, fall, or winter"))
    }

    // ── Entry point ───────────────────────────────────────────────────────────

    pub fn run() {
        let args = Args::parse();

        if args.day < 1 || args.day > 28 {
            eprintln!("error: --day must be 1–28");
            std::process::exit(1);
        }
        if args.year < 1 {
            eprintln!("error: --year must be ≥1");
            std::process::exit(1);
        }

        let platform_label = match args.platform {
            Platform::PC => "PC",
            Platform::Switch => "Switch",
        };

        println!();
        println!(
            "  {} seed {} — starting Year {} {} Day {} — {} day(s)",
            platform_label,
            args.seed,
            args.year,
            season_name(args.season),
            args.day,
            args.count,
        );
        println!("  {}", "─".repeat(75));
        println!("  {:<14}  {:<10}  {}", "Day", "d-roll", "Quest");
        println!("  {}", "─".repeat(75));

        let mut day = args.day;
        let mut season = args.season;
        let mut year = args.year;

        for _ in 0..args.count {
            let days_played = (year - 1) * 112 + season.index() * 28 + day as u32;
            let is_monday = matches!(day, 1 | 8 | 15 | 22);

            let mut type_rng = create_day_save_random(
                args.platform,
                days_played,
                args.seed,
                100.0,
                days_played as f64 * 777.0,
                0.0,
            )
            .expect("create_day_save_random failed");

            let d = type_rng.gen_float().expect("gen_float failed");

            let day_label = format!(
                "Y{} {} {:>2}{}",
                year,
                &season_name(season)[..3],
                day,
                if is_monday { " Mon" } else { "    " },
            );

            let quest = if days_played <= 1 {
                "(no quest — DaysPlayed ≤ 1)".to_string()
            } else {
                describe_quest(args.platform, args.seed, days_played, season, d, is_monday)
            };

            println!("  {day_label}  {d:.6}  {quest}");

            // Advance one calendar day.
            if day < 28 {
                day += 1;
            } else {
                day = 1;
                season = match season {
                    Season::Spring => Season::Summer,
                    Season::Summer => Season::Fall,
                    Season::Fall => Season::Winter,
                    Season::Winter => {
                        year += 1;
                        Season::Spring
                    }
                };
            }
        }

        println!();
    }

    // ── Quest description ─────────────────────────────────────────────────────

    fn describe_quest(
        platform: Platform,
        seed: u64,
        days_played: u32,
        season: Season,
        d: f64,
        is_monday: bool,
    ) -> String {
        if d < 0.08 {
            describe_resource(platform, seed, days_played)
        } else if d < 0.2 {
            if days_played > 5 {
                "SlayMonster  (or None if mine not yet entered)".to_string()
            } else {
                "None".to_string()
            }
        } else if d < 0.5 {
            "None".to_string()
        } else if d < 0.6 {
            describe_fishing(platform, seed, days_played, season)
        } else if d < 0.66 && is_monday {
            "Socialize  (or ItemDelivery if already active)".to_string()
        } else {
            "ItemDelivery".to_string()
        }
    }

    fn describe_resource(platform: Platform, seed: u64, days_played: u32) -> String {
        let mut rng = create_initialisation_random(platform, seed, days_played)
            .expect("create_initialisation_random failed");

        let resource_type = rng.gen_range(0..6).expect("gen_range failed") * 2;
        let items = resource_type_to_items(resource_type);

        let item_strs: Vec<String> = items
            .iter()
            .map(|&id| {
                OBJECTS
                    .get(&id)
                    .map(|o| format!("{} ({})", o.name, id))
                    .unwrap_or_else(|| format!("ID {id}"))
            })
            .collect();

        let item_display = if item_strs.len() == 1 {
            item_strs[0].clone()
        } else {
            format!("{} if mine>lv40, else {}", item_strs[0], item_strs[1])
        };

        format!("ResourceCollection  [type {resource_type}] → {item_display}")
    }

    fn describe_fishing(
        platform: Platform,
        seed: u64,
        days_played: u32,
        season: Season,
    ) -> String {
        let mut rng = create_initialisation_random(platform, seed, days_played)
            .expect("create_initialisation_random failed");

        let sub_pool = rng.next_bool().expect("next_bool failed");
        let npc = if sub_pool { "Willy" } else { "Demetrius" };

        let pool = FISHING_POOLS[season.index() as usize][sub_pool as usize];
        let fish_idx = rng.gen_range(0..pool.len() as i32).expect("gen_range failed") as usize;
        let fish_id = pool[fish_idx];

        let fish_name = OBJECTS
            .get(&fish_id)
            .map(|o| o.name)
            .unwrap_or("(unknown)");

        format!("Fishing  [{npc}] → {fish_name} ({fish_id})")
    }

    fn season_name(s: Season) -> &'static str {
        match s {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        }
    }
}
