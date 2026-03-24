use stardew_seed_cracker::codegen::ELIGIBLE_OBJECTS;
use stardew_seed_cracker::traveling_merchant::{generate_stock, Platform};
use stardew_seed_cracker::xxhash::shop_seed;

fn main() {
    let mut uid: u64 = 0;
    let mut platform = Platform::PC;
    let mut day: i32 = 1;
    let mut season: u8 = 0; // 0=Spring 1=Summer 2=Fall 3=Winter
    let mut year: i32 = 1;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                i += 1;
                uid = args[i].parse().expect("--seed must be a non-negative integer");
            }
            "--platform" => {
                i += 1;
                platform = match args[i].to_lowercase().as_str() {
                    "switch" => Platform::Switch,
                    "pc" => Platform::PC,
                    other => panic!("--platform must be PC or Switch, got {:?}", other),
                };
            }
            "--day" => {
                i += 1;
                day = args[i].parse().expect("--day must be 1-28");
                assert!((1..=28).contains(&day), "--day must be 1-28");
            }
            "--season" => {
                i += 1;
                season = match args[i].to_lowercase().as_str() {
                    "spring" => 0,
                    "summer" => 1,
                    "fall" | "autumn" => 2,
                    "winter" => 3,
                    other => panic!("--season must be Spring/Summer/Fall/Winter, got {:?}", other),
                };
            }
            "--year" => {
                i += 1;
                year = args[i].parse().expect("--year must be a positive integer");
                assert!(year >= 1, "--year must be >= 1");
            }
            other => panic!("unknown argument {:?}", other),
        }
        i += 1;
    }

    // Stardew days_played: 1-indexed, 28 days/season, 4 seasons/year.
    let days_played = (year - 1) * 112 + season as i32 * 28 + day;
    let seed = shop_seed(days_played, uid);
    let stock = generate_stock(platform, seed);

    let season_name = ["Spring", "Summer", "Fall", "Winter"][season as usize];
    let platform_name = match platform {
        Platform::PC => "PC",
        Platform::Switch => "Switch",
    };

    println!("Platform  : {}", platform_name);
    println!("Date      : Day {} of {}, Year {} (days_played={})", day, season_name, year, days_played);
    println!("UID       : {}", uid);
    println!("Shop seed : {}", seed);
    println!();
    println!("{:<4}  {:<32}  {:>7}  {:>3}", "Slot", "Item", "Price", "Qty");
    println!("{}", "-".repeat(55));

    for (slot, item) in stock.items.iter().enumerate() {
        let obj = &ELIGIBLE_OBJECTS[item.eligible_index as usize];
        println!(
            "{:<4}  {:<32}  {:>6}g  {:>3}x",
            slot, obj.name, item.price, item.quantity
        );
    }
}
