use std::cmp::{max, min, Ordering};
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::io;
use std::time::Instant;

use crate::Action::{Brew, Cast, Learn, Rest, Wait};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn run() {

//    let mut brewed_potion_count = 0;
//    let mut opp_prev_score = 0;
//    let mut opp_brewed_count = 0;

    // game loop
    loop {
        let mut game: GameState = GameState {
            my_rupees: 0,
            opp_rupees: 0,
            my_ingredients: [0; 4],
            opp_ingredients: [0; 4],
            potions: BinaryHeap::new(),
            my_cast: Vec::new(),
            opp_cast: Vec::new(),
            tome_spells: Vec::new(),
            //unbrewable_potion_ids: HashSet::new(),
            my_disabled_spells: HashSet::new(),
        };

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play

        for _ in 0..action_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let action_id = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
            let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
            let delta_0 = parse_input!(inputs[2], i32); // tier-0 ingredient change
            let delta_1 = parse_input!(inputs[3], i32); // tier-1 ingredient change
            let delta_2 = parse_input!(inputs[4], i32); // tier-2 ingredient change
            let delta_3 = parse_input!(inputs[5], i32); // tier-3 ingredient change
            let price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
            let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax; For brews, this is the value of the current urgency bonus
            let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell; For brews, this is how many times you can still gain an urgency bonus
            let castable = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
            let repeatable = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

            let delta = [delta_0, delta_1, delta_2, delta_3];

            match &action_type[..] {
                "BREW" => game.potions.push(Potion {
                    id: action_id,
                    delta,
                    price,
                }),
                "CAST" => {
                    game.my_cast.push(Spell {
                        id: action_id,
                        delta,
                        read_ahead_tax: tome_index,
                        tax_count,
                        castable: castable == 1,
                        repeatable: repeatable == 1,
                    });

                    if castable != 1 {
                        game.my_disabled_spells.insert(action_id);
                    }
                },
                "OPPONENT_CAST" => game.opp_cast.push(Spell {
                    id: action_id,
                    delta,
                    read_ahead_tax: tome_index,
                    tax_count,
                    castable: castable == 1,
                    repeatable: repeatable == 1,
                }),
                "LEARN" => game.tome_spells.push(Spell {
                    id: action_id,
                    delta,
                    read_ahead_tax: tome_index,
                    tax_count,
                    castable: castable == 1,
                    repeatable: repeatable == 1,
                }),
                _ => {}
            }
        }

        for i in 0..2 as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let inv_0 = parse_input!(inputs[0], i32); // tier-0 ingredients in inventory
            let inv_1 = parse_input!(inputs[1], i32);
            let inv_2 = parse_input!(inputs[2], i32);
            let inv_3 = parse_input!(inputs[3], i32);
            let score = parse_input!(inputs[4], i32); // amount of rupees

            let ingredients = [inv_0, inv_1, inv_2, inv_3];

            if i == 0 {
                game.my_ingredients = ingredients;
                game.my_rupees = score;
            } else {
                game.opp_ingredients = ingredients;
                game.opp_rupees = score;

//                if score > opp_prev_score {
//                    opp_prev_score = score;
//                    opp_brewed_count += 1;
//                }
            }
        }

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");


        // in the first league: BREW <id> | WAIT; later: BREW <id> | CAST <id> [<times>] | LEARN <id> | REST | WAIT

//        for potion in &game.potions {
//            if let None = pay(&potion.delta, &game.my_ingredients) {
//                game.unbrewable_potion_ids.insert(potion.id);
//            }
//        }

        let (action, score) = get_best_action(&game);

//        if let Some(best_potion) = get_best_brewable_potion(&game) {
//            if (best_potion.price * 5) as f32 > score * 0.5 ||
//                best_potion.price > 10 ||
//                max(brewed_potion_count, opp_brewed_count) >= 4 ||
//                game.opp_rupees - game.my_rupees > 15 {
//
//                brewed_potion_count += 1;
//                println!("BREW {}", best_potion.id);
//                continue;
//            }
//        } else {
//            eprintln!("Can't brew any potions.");
//        }

        match action {
            Brew(id) => println!("BREW {}", id),
            Cast(id, times) => println!("CAST {} {}", id, times),
            Learn(id) => println!("LEARN {}", id),
            Wait => println!("WAIT"),
            Rest => println!("REST"),
        }
    }
}

struct GameState {
    my_rupees: i32,
    opp_rupees: i32,
    my_ingredients: [i32; 4],
    opp_ingredients: [i32; 4],
    potions: BinaryHeap<Potion>,
    my_cast: Vec<Spell>,
    opp_cast: Vec<Spell>,
    tome_spells: Vec<Spell>,
    //unbrewable_potion_ids: HashSet<i32>,
    my_disabled_spells: HashSet<i32>,
}

#[derive(Debug, Eq, PartialEq)]
struct Potion {
    id: i32,
    delta: [i32; 4],
    price: i32,
}

impl Ord for Potion {
    fn cmp(&self, other: &Potion) -> Ordering {
        self.price.cmp(&other.price)
    }
}

impl PartialOrd for Potion {
    fn partial_cmp(&self, other: &Potion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
struct Spell {
    id: i32,
    delta: [i32; 4],
    read_ahead_tax: i32,
    tax_count: i32,
    castable: bool,
    repeatable: bool,
}

#[derive(Debug, Copy, Clone)]
enum Action {
    Wait,
    Brew(i32),
    Cast(i32, i32),
    Learn(i32),
    Rest,
}

#[derive(Debug)]
struct State {
    new_spells: HashSet<i32>,
    ingredients: [i32; 4],
    disabled_spells: HashSet<i32>,
    brewed_potions: HashSet<i32>,
    depth: i32,
    root_action: Action,
    cumulative_score: f32,
}
//
//fn get_best_brewable_potion(game: &GameState) -> Option<&Potion> {
//    for potion in &game.potions {
//        if let Some(_) = pay(&potion.delta, &game.my_ingredients) {
//            eprintln!("Can brew potion {} with delta {:?}. Price is {} and my inv: {:?}", potion.id, potion.delta, potion.price, game.my_ingredients);
//            return Some(potion);
//        }
//    }
//
//    None
//}

fn get_best_action(game: &GameState) -> (Action, f32) {
    let time = Instant::now();

    let state = State {
        ingredients: game.my_ingredients,
        new_spells: HashSet::new(),
        disabled_spells: game.my_disabled_spells.clone(),
        depth: 0,
        root_action: Action::Wait,
        cumulative_score: 0.0,
        brewed_potions: HashSet::new(),
    };

    let mut queue = VecDeque::new();
    queue.push_back(state);

    let mut node_count = 0;
    let mut max_depth = 0;
    let mut max_width = 0;
    let mut best = (Wait, std::f32::MIN);

    while let Some(current_state) = queue.pop_front() {
        let score = (score(&current_state, game) + current_state.cumulative_score);// / (current_state.depth + 1 )as f32;
        //eprintln!("Score= {} for state: {:#?}", score, current_state);

        if current_state.depth > 0 && best.1 < score {
            best = (current_state.root_action, score);
        }

        node_count += 1;
        max_depth = max(max_depth, current_state.depth);

        if is_timeout(&time) {
            eprintln!("TIMEOUT. Depth: {}, Width: {}, Nodes: {}", max_depth, max_width, node_count);
            break;
        }

        let mut width = 0;
        for child in get_children(&current_state, game, score, &time) {
            width += 1;
            queue.push_back(child)
        }

        if width > max_width {
            max_width = width;
        }
    }

    eprintln!("Search Complete. Depth: {}, Width: {}, Nodes: {}. Best: {:?}", max_depth, max_width, node_count, best);
    best
}

fn score(state: &State, game: &GameState) -> f32 {
    let mut score = 0.0;

//    for i in 1..5 as i32 {
//        score += ((i * 2) as f32 - 1.8) + state.ingredients[(i - 1) as usize] as f32;
//    }

    for potion in &game.potions {
        if !state.brewed_potions.contains(&potion.id) {
            for i in 0..4 {
                score += ((state.ingredients[i] + potion.delta[i]) as f32 * potion.price as f32 * ((i + 1) * 3) as f32) / 5.0;
            }
            continue;
        }

        score += potion.price as f32 * 80.0
//        if !game.unbrewable_potion_ids.contains(&potion.id) {
//            continue;
//        }
//
        //eprintln!("Score brew");
//        if let Some(_) = pay(&potion.delta, &state.ingredients) {
//            score += potion.price as f32 / 5.0; //5 total potions
//        }
    }

    score
}

fn get_children(state: &State, game: &GameState, parent_score: f32, time: &Instant) -> Vec<State> {
    let mut new_states = Vec::new();
    let new_score = parent_score;
    let new_depth = state.depth + 1;

    if !state.disabled_spells.is_empty() {
        new_states.push(State {
            ingredients: state.ingredients,
            new_spells: state.new_spells.clone(),
            disabled_spells: HashSet::new(),
            depth: new_depth,
            cumulative_score: new_score,
            brewed_potions: state.brewed_potions.clone(),
            root_action: match state.root_action {
                Wait => Rest,
                _ => state.root_action,
            },
        });
    }

    for potion in &game.potions {
        if is_timeout(&time) {
            break;
        }

        if state.brewed_potions.contains(&potion.id) {
            continue;
        }

        if let Some(new_ingredients) = pay(&potion.delta, &state.ingredients) {
            let mut brewed_potions = state.brewed_potions.clone();
            brewed_potions.insert(potion.id);

            new_states.push(State {
                ingredients: new_ingredients,
                new_spells: state.new_spells.clone(),
                disabled_spells: state.disabled_spells.clone(),
                depth: new_depth,
                cumulative_score: new_score,
                brewed_potions,
                root_action: match state.root_action {
                    Wait => Brew(potion.id),
                    _ => state.root_action,
                },
            });
        }
    }

    for spell in &game.my_cast {
        if is_timeout(&time) {
            break;
        }

        if !spell.castable || state.disabled_spells.contains(&spell.id) {
            continue;
        }

        for times in 1..3 {
            let delta = match spell.repeatable {
                true => [spell.delta[0] * times, spell.delta[1] * times, spell.delta[2] * times, spell.delta[3] * times],
                false => spell.delta,
            };

            if let Some(new_ingredients) = pay(&delta, &state.ingredients) {
                let mut disabled = state.disabled_spells.clone();
                disabled.insert(spell.id);

                new_states.push(State {
                    ingredients: new_ingredients,
                    new_spells: state.new_spells.clone(),
                    disabled_spells: disabled,
                    depth: new_depth,
                    cumulative_score: new_score,
                    brewed_potions: state.brewed_potions.clone(),
                    root_action: match state.root_action {
                        Wait => Cast(spell.id, times),
                        _ => state.root_action,
                    },
                });
            }

            if !spell.repeatable {
                break;
            }
        }
    }

    for spell in &game.tome_spells {
        if is_timeout(&time) {
            break;
        }

        if state.disabled_spells.contains(&spell.id) {
            continue;
        }

        if state.new_spells.contains(&spell.id) {
            //Castable
            for times in 1..3 {
                let delta = match spell.repeatable {
                    true => [spell.delta[0] * times, spell.delta[1] * times, spell.delta[2] * times, spell.delta[3] * times],
                    false => spell.delta,
                };

                if let Some(new_ingredients) = pay(&delta, &state.ingredients) {
                    let mut disabled = state.disabled_spells.clone();
                    disabled.insert(spell.id);

                    new_states.push(State {
                        ingredients: new_ingredients,
                        new_spells: state.new_spells.clone(),
                        disabled_spells: disabled,
                        depth: new_depth,
                        cumulative_score: new_score,
                        brewed_potions: state.brewed_potions.clone(),
                        root_action: state.root_action, //Special case since it was already not castable. i.e not original
                    });
                }

                if !spell.repeatable {
                    break;
                }
            }

            continue;
        }

        //Learn
        if let Some(new_ingredients) = pay(&[-spell.read_ahead_tax, 0, 0, 0], &state.ingredients).as_mut() {
            let mut new_spells = state.new_spells.clone();
            new_spells.insert(spell.id);

            let total = new_ingredients[0] + new_ingredients[1] + new_ingredients[2] + new_ingredients[3];
            new_ingredients[0] += min(spell.tax_count, 10 - total);

            new_states.push(State {
                ingredients: *new_ingredients,
                new_spells,
                disabled_spells: state.disabled_spells.clone(),
                depth: new_depth,
                cumulative_score: new_score,
                brewed_potions: state.brewed_potions.clone(),
                root_action: match state.root_action {
                    Wait => Learn(spell.id),
                    _ => state.root_action,
                },
            });
        }
    }

    new_states
}

fn pay(cost: &[i32; 4], money: &[i32; 4]) -> Option<[i32; 4]> {
    let mut result = [0; 4];
    let mut total = 0;

    for i in 0..4 {
        result[i] = money[i] + cost[i];

        if result[i] < 0 {
            return None;
        }

        total += result[i];
    }

    if total > 10 {
        return None
    }

    Some(result)
}

fn is_timeout(time: &Instant) -> bool {
    time.elapsed().as_millis() >= 41
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
    //println!("{:?}", pay([0;4], [1;4]));
    run();
}
