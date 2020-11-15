use crate::solution::execution::Game;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

mod solution {
    pub mod execution {
        use std::io;
        use std::collections::{HashMap, HashSet, VecDeque};
        use crate::solution::models::{Order, ActionType};
        use crate::solution::models::Spell;
        use crate::solution::models::State;
        use crate::solution::models::Action;
        use crate::solution::runtime::{ActionsRepository, ActionExecutor};
        use std::cmp::{min};

        pub struct Game;

        impl Game {
            pub fn run() {
                let mut repo = ActionsRepository::new(HashMap::new());

                // game loop
                loop {
                    //Define values
                    let mut actions: HashMap<i32, Box<dyn Action>> = HashMap::new();
                    let mut my_inactive_spells: HashSet<i32> = HashSet::new();
                    let mut my_ingredients: [i32;4] = [0;4];
                    let mut my_rupees: i32 = 0;

                    //Read input
                    let mut input_line = String::new();
                    io::stdin().read_line(&mut input_line).unwrap();
                    let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play
                    for _ in 0..action_count as usize {
                        let mut input_line = String::new();
                        io::stdin().read_line(&mut input_line).unwrap();
                        let inputs = input_line.split(" ").collect::<Vec<_>>();
                        let action_id = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
                        let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
                        let action_type:&str = &action_type[..];
                        let delta_0 = parse_input!(inputs[2], i32); // tier-0 ingredient change
                        let delta_1 = parse_input!(inputs[3], i32); // tier-1 ingredient change
                        let delta_2 = parse_input!(inputs[4], i32); // tier-2 ingredient change
                        let delta_3 = parse_input!(inputs[5], i32); // tier-3 ingredient change
                        let price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
                        let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax
                        let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell
                        let castable = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
                        let repeatable = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

                        match action_type {
                            "OPPONENT_CAST" => {
                                //eprintln!("Skipping action {} since it is an opponent spell", action_id);
                            },
                            "CAST" => {
                                //eprintln!("Adding action {} to my spells", action_id);

                                actions.insert(action_id, Box::new(Spell::new(
                                    action_id,
                                    [delta_0, delta_1, delta_2, delta_3]
                                )));

                                if castable != 1 {
                                    my_inactive_spells.insert(action_id);
                                }
                            },
                            "BREW" => {
                                actions.insert(action_id, Box::new(Order::new(
                                    action_id,
                                    price,
                                    [delta_0, delta_1, delta_2, delta_3])));
                            },
                            _ => eprintln!("Unrecognized action type: {}", action_type),
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

                        match i {
                            0 => {
                                my_ingredients = [inv_0, inv_1, inv_2, inv_3];
                                my_rupees = score;
                            },
                            1 => {
                                //eprintln!("Skipping opponent properties...");
                            },
                            _ => eprintln!("Unrecognized data index: {}", i),
                        }
                    }

                    let state = State::new(
                        my_ingredients,
                        my_rupees,
                        my_inactive_spells,
                        None,
                        0);

                    repo.reset(actions);

                    eprintln!("State: {:#?}", state);

                    let best_action = Game::search(state, &repo);
                    eprintln!("Best Action: {}", best_action);

                    // in the first league: BREW <id> | WAIT; later: BREW <id> | CAST <id> [<times>] | LEARN <id> | REST | WAIT
                    if let Some(action) = repo.get_action(&best_action) {
                        match action.get_action_type() {
                            ActionType::Spell => println!("CAST {}", best_action),
                            ActionType::Order => println!("BREW {}", best_action),
                        }
                    }
                    else {
                        println!("REST")
                    }
                }
            }

            fn search(state: State, repo: &ActionsRepository) -> i32 {
                let mut queue: VecDeque<State> = VecDeque::new();
                queue.push_back(state);

                let mut best: (f32, i32) = (std::f32::MIN, 0); //(score, action_id)

                while !queue.is_empty() {
                    let current_state = queue.pop_front().unwrap();
                    let score = Game::score(&current_state, repo);
                    let root_action_id = current_state.get_root_action_id().unwrap_or(0);

                    eprintln!("Searching state with root id: {} at depth {}. Score is {}", root_action_id, current_state.get_depth(), score);

                    if root_action_id != 0 {
                        eprintln!("Action Type: {:?}", repo.get_action(&root_action_id).unwrap().get_action_type());
                    }

                    if score > best.0 {
                        eprintln!("New best. Action: {}, Score: {}", root_action_id, score);
                        best = (score, root_action_id);
                    }

                    for child in Game::get_children(&current_state, repo) {
                        if child.get_depth() > 10 {
                            eprintln!("Truncating search");
                            continue;
                        }

                        queue.push_back(child)
                    }
                }

                best.1
            }

            fn get_children(state: &State, repo: &ActionsRepository) -> Vec<State> {
                let mut new_states: Vec<State> = Vec::new();

                for action in repo.get_actions() {
                    if !state.is_action_active(action) {
                        continue;
                    }

                    if let Some(new_state) = ActionExecutor::execute(repo, state, action) {
                        new_states.push(new_state);
                    }
                }

                new_states
            }

            fn score(state: &State, repo: &ActionsRepository) -> f32 {
                //eprintln!("Scoring state: {:#?}", state);

                // Having rupees is the best place to be :). We reward rupees 10 times
                let mut score: f32 = (state.get_rupees() * 10) as f32;
                let ingredients = state.get_ingredients();

                for action_id in repo.get_actions() {
                    let action = repo.get_action(action_id).unwrap();

                    if let ActionType::Spell = action.get_action_type() {
                        continue;
                    }

                    let order: &Order = action.as_any().downcast_ref::<Order>().unwrap();
                    let order_requirement = order.get_ingredient_change();
                    let half_price = order.get_price() as f32 / 2.0;

                    //eprintln!("Adding score for order: {:#?}", order);

                    let order_distance = min(ingredients[0] + order_requirement[0], 0) +
                        min(ingredients[1] + order_requirement[1], 0) +
                        min(ingredients[2] + order_requirement[2], 0) +
                        min(ingredients[3] + order_requirement[3], 0);

                    //eprintln!("Order distance: {}", order_distance);

                    if order_distance >= 0 {
                        // We're rewarding being able to brew with half the price of the order. i.e if we can brew 2 orders. go with the one that pays more.
                        score += half_price;
                    }

                    // We're punishing not being able to produce orders with half the distance.
                    // We can multiply by the order price to have the price of the order influence what spells to cast.
                    // i.e if we can cast 2 spells that will open up 2 orders, open the one that pays more
                    score += order_distance as f32 / 2.0 * (half_price / 2.0);
                }

                //eprintln!();
                score
            }
        }
    }

    /// Models Module
    pub mod models {
        use std::collections::{HashSet};
        use std::any::Any;

        #[derive(Debug)]
        pub enum ActionType {
            Spell,
            Order
        }

        pub trait Action {
            fn get_id(&self) -> &i32;
            fn get_ingredient_change(&self) -> &[i32;4];
            fn get_action_type(&self) -> ActionType;
            fn as_any(&self) -> &dyn Any;
        }

        #[derive(Debug, Copy, Clone)]
        pub struct Spell {
            id: i32,
            ingredient_change: [i32;4],
        }

        impl Spell {
            pub fn new(id: i32, ingredient_change: [i32;4]) -> Spell {
                Spell {
                    id,
                    ingredient_change,
                }
            }
        }

        impl Action for Spell {
            fn get_id(&self) -> &i32 {
                &self.id
            }

            fn get_ingredient_change(&self) -> &[i32;4] {
                &self.ingredient_change
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Spell
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        #[derive(Debug, Copy, Clone)]
        pub struct Order {
            id: i32,
            ingredient_change: [i32;4],
            price: i32,
        }

        impl Order {
            pub fn new(id: i32, price: i32, ingredient_change: [i32;4]) -> Order {
                Order {
                    id,
                    price,
                    ingredient_change,
                }
            }

            pub fn get_price(&self) -> i32 {
                self.price
            }
        }

        impl Action for Order {
            fn get_id(&self) -> &i32 {
                &self.id
            }

            fn get_ingredient_change(&self) -> &[i32;4] {
                &self.ingredient_change
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Order
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        #[derive(Debug, Clone)]
        pub struct State {
            my_ingredients: [i32;4],
            my_rupees: i32,
            inactive_actions: HashSet<i32>,
            root_action_id: Option<i32>,
            depth: i32,
        }

        impl State {
            pub fn new(
                my_ingredients: [i32;4],
                my_rupees: i32,
                inactive_actions: HashSet<i32>,
                root_action_id: Option<i32>,
                depth: i32) -> State {

                State {
                    my_ingredients,
                    my_rupees,
                    inactive_actions,
                    root_action_id,
                    depth
                }
            }

            pub fn get_ingredients(&self) -> &[i32;4] {
                &self.my_ingredients
            }

            pub fn get_rupees(&self) -> i32 {
                self.my_rupees
            }

            pub fn get_inactive_actions(&self) -> &HashSet<i32> {
                &self.inactive_actions
            }

            pub fn get_root_action_id(&self) -> Option<i32> {
                self.root_action_id
            }

            pub fn get_depth(&self) -> i32 {
                self.depth
            }

            pub fn is_action_active(&self, action_id: &i32) -> bool {
                !self.inactive_actions.contains(action_id)
            }

            pub fn deactivate_action(&mut self, action_id: &i32) {
                self.inactive_actions.insert(action_id.clone());
            }
        }
    }

    /// Runtime Module
    pub mod runtime {
        use crate::solution::models::{Action, State, ActionType, Order};
        use std::collections::HashMap;

        pub struct ActionsRepository {
            actions: Option<HashMap<i32, Box<dyn Action>>>
        }

        impl ActionsRepository {
            pub fn new(actions: HashMap<i32, Box<dyn Action>>) -> ActionsRepository {
                ActionsRepository {
                    actions: Some(actions),
                }
            }

            pub fn reset(&mut self, actions: HashMap<i32, Box<dyn Action>>) {
                self.actions = Some(actions);
            }

            pub fn get_action(&self, id: &i32) -> Option<&Box<dyn Action>> {
                if let Some(action) = self.actions.as_ref().unwrap().get(id) {
                    return Some(action);
                }

                None
            }

            pub fn get_actions(&self) -> Vec<&i32> {
                self.actions.as_ref().unwrap().keys().collect()
            }
        }

        /// Executes actions
        pub struct ActionExecutor;

        impl ActionExecutor {
            pub fn execute(repo: &ActionsRepository, state: &State, action_id: &i32) -> Option<State> {
                let state = state;

                let action: Option<&Box<dyn Action>> = repo.get_action(action_id);

                if action.is_none() {
                    return None;
                }

                let action = action.unwrap();

                let current_ingredients = state.get_ingredients();
                let ingredient_change = action.get_ingredient_change();

                let new_ingredients = [
                    current_ingredients[0] + ingredient_change[0],
                    current_ingredients[1] + ingredient_change[1],
                    current_ingredients[2] + ingredient_change[2],
                    current_ingredients[3] + ingredient_change[3]];

                for i in 0..new_ingredients.len() {
                    if new_ingredients[i] < 0 {
                        //eprintln!("Insufficient funds for action: {}", action_id);
                        return None;
                    }
                }

                let mut new_rupees = state.get_rupees();

                //Action specific customization
                match action.get_action_type() {
                    ActionType::Order => {
                        let order: &Order = action.as_any().downcast_ref::<Order>().unwrap();
                        new_rupees += order.get_price();
                    },
                    _ => {},
                }

                let root_action_id = match state.get_root_action_id() {
                    Some(id) => id,
                    None => action_id.clone(),
                };

                let mut new_state = State::new(
                    new_ingredients,
                    new_rupees,
                    state.get_inactive_actions().clone(),
                    Some(root_action_id),
                    state.get_depth() + 1);

                new_state.deactivate_action(action_id);

                Some(new_state)
            }
        }
    }
}

fn main() {
    Game::run();
}
