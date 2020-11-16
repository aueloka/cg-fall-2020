use crate::solution::execution::Game;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

const TIMEOUT: u128 = 50;
const MAX_DEPTH: i32 = 5;
const NULL_ACTION_ID: i32 = -1;
const INGREDIENT_TIER_COUNT: usize = 4;
const REST_ID: i32 = -50;
const MAX_INGREDIENT_COUNT: i32 = 10;
static NO_INGREDIENT_CHANGE: [i32; INGREDIENT_TIER_COUNT] = [0; 4];

mod solution {
    pub mod execution {
        use std::collections::HashSet;
        use std::io;

        use crate::INGREDIENT_TIER_COUNT;
        use crate::solution::models::{ActionType, Order, UnlearntSpell};
        use crate::solution::models::LearntSpell;
        use crate::solution::models::State;
        use crate::solution::runtime::{ActionsRepository, Solver};

        pub struct Game;

        impl Game {
            pub fn run() {
                // game loop
                loop {
                    let (repo, my_inactive_spells) = Game::read_actions();
                    let state = Game::read_state(my_inactive_spells);
                    let best_action = Solver::search(state, &repo);

                    // in the first league: BREW <id> | WAIT; later: BREW <id> | CAST <id> [<times>] | LEARN <id> | REST | WAIT
                    if let Some(action) = repo.get_action(&best_action) {
                        match action.get_action_type() {
                            ActionType::Cast => println!("CAST {} {}", &best_action, 1 /*We only cast once for now. TODO: Evaluate how to choose the best times*/),
                            ActionType::Brew => println!("BREW {}", &best_action),
                            ActionType::Learn => println!("LEARN {}", &best_action),
                            ActionType::Rest => println!("REST"),
                        }

                        continue;
                    }

                    //No action was found.
                    println!("WAIT");
                }
            }

            fn read_actions() -> (ActionsRepository, HashSet<i32>) {
                let mut repo = ActionsRepository::new();
                repo.add_rest();

                //Define values
                let mut my_inactive_spells: HashSet<i32> = HashSet::new();

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
                    let action_type: &str = &action_type[..];
                    let delta_0 = parse_input!(inputs[2], i32); // tier-0 ingredient change
                    let delta_1 = parse_input!(inputs[3], i32); // tier-1 ingredient change
                    let delta_2 = parse_input!(inputs[4], i32); // tier-2 ingredient change
                    let delta_3 = parse_input!(inputs[5], i32); // tier-3 ingredient change
                    let price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
                    let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax
                    let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell
                    let castable = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
                    //let repeatable = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

                    match action_type {
                        "OPPONENT_CAST" => {},
                        "CAST" => {
                            repo.add_learnt_spell(action_id, Box::new(LearntSpell::new(
                                action_id,
                                [delta_0, delta_1, delta_2, delta_3],
                                //repeatable == 1,
                            )));

                            if castable != 1 {
                                my_inactive_spells.insert(action_id);
                            }
                        },
                        "LEARN" => {
                            repo.add_unlearnt_spell(action_id, Box::new(UnlearntSpell::new(
                                action_id,
                                [delta_0, delta_1, delta_2, delta_3],
                                //repeatable == 1,
                                tome_index,
                                tax_count,
                            )));
                        },
                        "BREW" => {
                            repo.add_order(action_id, Box::new(Order::new(
                                action_id,
                                price,
                                [delta_0, delta_1, delta_2, delta_3])));
                        }
                        _ => eprintln!("Unrecognized action type: {}", action_type),
                    }
                }

                (repo, my_inactive_spells)
            }

            fn read_state(my_inactive_spells: HashSet<i32>) -> State {
                let mut my_ingredients: [i32; INGREDIENT_TIER_COUNT] = [0; INGREDIENT_TIER_COUNT];
                let mut my_rupees: i32 = 0;

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
                        }
                        1 => {
                            //eprintln!("Skipping opponent properties...");
                        }
                        _ => eprintln!("Unrecognized data index: {}", i),
                    }
                }

                State::new(
                    my_ingredients,
                    my_rupees,
                    HashSet::new(),
                    my_inactive_spells,
                    HashSet::new(),
                    None,
                    0)
            }
        }
    }

    /// Runtime Module
    pub mod runtime {
        use std::cmp::min;
        use std::collections::{HashMap, HashSet, VecDeque};
        use std::time::Instant;

        use crate::{INGREDIENT_TIER_COUNT, MAX_DEPTH, MAX_INGREDIENT_COUNT, NULL_ACTION_ID, REST_ID, TIMEOUT};
        use crate::solution::models::{Action, ActionType, LearntSpell, Order, Rest, State, UnlearntSpell};

        pub struct ActionsRepository {
            actions: HashMap<i32, Box<dyn Action>>,
            orders: Vec<i32>,
        }

        impl ActionsRepository {
            pub fn new() -> ActionsRepository {
                ActionsRepository {
                    actions: HashMap::new(),
                    orders: Vec::new(),
                }
            }

            pub fn add_order(&mut self, id: i32, order: Box<Order>) {
                self.actions.insert(id, order);
                self.orders.push(id);
            }

            pub fn add_learnt_spell(&mut self, id: i32, spell: Box<LearntSpell>) {
                self.actions.insert(id, spell);
            }

            pub fn add_unlearnt_spell(&mut self, id: i32, spell: Box<UnlearntSpell>) {
                self.actions.insert(id, spell);
            }

            pub fn add_rest(&mut self) {
                self.actions.insert(REST_ID, Box::new(Rest::new()));
            }

            pub fn get_action(&self, id: &i32) -> Option<&Box<dyn Action>> {
                self.actions.get(id)
            }

            pub fn get_action_ids(&self) -> Vec<&i32> {
                self.actions.keys().collect()
            }

            pub fn get_order_ids(&self) -> &Vec<i32> {
                &self.orders
            }
        }

        /// Executes actions
        pub struct ActionExecutor;

        impl ActionExecutor {
            pub fn execute(repo: &ActionsRepository, state: &State, action_id: &i32) -> Option<State> {
                let state = state;

                let action: Option<&Box<dyn Action>> = repo.get_action(&action_id);

                if action.is_none() {
                    return None;
                }

                let action = action.unwrap();
                let root_action_id = state.get_root_action_id().unwrap_or(action_id.clone());

                let current_ingredients = state.get_ingredients();

                if action.is_rest() {
                    return Some(State::new(
                        [current_ingredients[0], current_ingredients[1], current_ingredients[2], current_ingredients[3]],
                        state.get_rupees().clone(),
                        state.get_inactive_orders().clone(),
                        HashSet::new(), //Reset inactive spells
                        state.get_learnt_spells().clone(),
                        Some(root_action_id),
                        state.get_depth() + 1));
                }

                let mut default_ingredient_change = [0; INGREDIENT_TIER_COUNT];

                let ingredient_change = match action.get_action_type() {
                    ActionType::Learn => {
                        if state.get_learnt_spells().contains(&action_id) {
                            //If we already learned, it works like a regular spell
                            action.get_ingredient_change()
                        } else {
                            let unlearnt_spell: &UnlearntSpell = action.as_any().downcast_ref::<UnlearntSpell>().unwrap();

                            //Only remove the read-ahead tax for learning
                            default_ingredient_change[0] = unlearnt_spell.get_read_ahead_tax() * -1;

                            &default_ingredient_change
                        }
                    }
                    _ => action.get_ingredient_change(),
                };

                let mut new_ingredients = [
                    current_ingredients[0] + ingredient_change[0],
                    current_ingredients[1] + ingredient_change[1],
                    current_ingredients[2] + ingredient_change[2],
                    current_ingredients[3] + ingredient_change[3]];

                let mut total_ingredients = 0;

                for i in 0..INGREDIENT_TIER_COUNT {
                    if new_ingredients[i] < 0 {
                        //Unaffordable action
                        return None;
                    }

                    total_ingredients += new_ingredients[i];
                }

                if total_ingredients > MAX_INGREDIENT_COUNT {
                    return None;
                }

                let mut new_rupees = state.get_rupees().clone();
                let mut learnt_spells = state.get_learnt_spells().clone();
                let mut is_new_learn = false;

                //Action specific customization
                match action.get_action_type() {
                    ActionType::Brew => {
                        //eprintln!("Evaluating brew");
                        let order: &Order = action.as_any().downcast_ref::<Order>().unwrap();
                        new_rupees += order.get_price();
                    },
                    ActionType::Learn => {
                        if !state.get_learnt_spells().contains(&action_id) {
                            //eprintln!("Evaluating learn");
                            let unlearnt_spell: &UnlearntSpell = action.as_any().downcast_ref::<UnlearntSpell>().unwrap();
                            new_ingredients[0] += min(MAX_INGREDIENT_COUNT - total_ingredients, unlearnt_spell.get_tax_gain());
                            learnt_spells.insert(action_id.clone());
                            is_new_learn = true;
                        }
                    },
                    _ => {},
                }

                let mut new_state = State::new(
                    new_ingredients,
                    new_rupees,
                    state.get_inactive_orders().clone(),
                    state.get_inactive_spells().clone(),
                    learnt_spells,
                    Some(root_action_id),
                    state.get_depth() + 1);

                match action.get_action_type() {
                    ActionType::Brew => new_state.deactivate_order(action_id),
                    ActionType::Rest => {},
                    _ => new_state.deactivate_spell(action_id, is_new_learn),
                }

                //eprintln!("New state: {:#?}", new_state);
                Some(new_state)
            }
        }

        pub struct Solver;

        impl Solver {
            pub fn search(state: State, repo: &ActionsRepository) -> i32 {
                let time = Instant::now();

                let mut queue: VecDeque<State> = VecDeque::new();
                queue.push_back(state);

                let mut best: (f32, i32) = (std::f32::MIN, NULL_ACTION_ID); //(score, action_id)

                let mut node_count = 0;

                while !queue.is_empty() {
                    let current_state = queue.pop_front().unwrap();

                    if time.elapsed().as_millis() >= TIMEOUT {
                        eprintln!("TIMEOUT. Depth: {}", &current_state.get_depth());
                        break;
                    }

                    let score = Solver::score(&current_state, repo);
                    let root_action_id = current_state.get_root_action_id().unwrap_or(NULL_ACTION_ID);

                    node_count += 1;

                    //eprintln!("Searching state with root id: {} at depth {}. Score is {}", root_action_id, current_state.get_depth(), score);
//                    if let Some(action) = repo.get_action(&root_action_id) {
//                        eprintln!("Id: {}; Action type: {:?}; Score: {}, Depth: {}", root_action_id, action.get_action_type(), score, current_state.get_depth());
//                    }

                    if score > best.0 {
                        //eprintln!("New best. Action: {}, Score: {}", root_action_id, score);
                        best = (score, root_action_id);
                    }

                    if current_state.get_depth() >= &MAX_DEPTH {
                        //eprintln!("Max depth reached.");
                        continue;
                    }

                    for child in Solver::get_children(&current_state, repo, &time) {
                        queue.push_back(child)
                    }
                }

                eprintln!("Evaluated {} nodes.", &node_count);
                best.1
            }

            fn get_children(state: &State, repo: &ActionsRepository, time: &Instant) -> Vec<State> {
                let mut new_states: Vec<State> = Vec::new();

                for action in repo.get_action_ids() {
                    if !state.is_action_active(&action) {
                        //eprintln!("Inactive action: {}. State: {:#?}", action, state);
                        continue;
                    }

                    if time.elapsed().as_millis() >= TIMEOUT - 1 {
                        break;
                    }

                    if let Some(new_state) = ActionExecutor::execute(repo, state, action) {
                        new_states.push(new_state);
                    }
                }

                //eprintln!("Branches for state at depth: {}, {}", state.get_depth(), new_states.len());
                new_states
            }

            fn score(state: &State, repo: &ActionsRepository) -> f32 {
                //eprintln!("Scoring state: {:#?}", state);

                // Having rupees is the best place to be :). We reward rupees 10 times
                let mut score: f32 = (state.get_rupees() * 10) as f32;

                let ingredients = state.get_ingredients();

                for order_id in repo.get_order_ids() {
                    if state.get_inactive_orders().contains(order_id) {
                        continue;
                    }

                    let action = repo.get_action(order_id).unwrap();
                    let order: &Order = action.as_any().downcast_ref::<Order>().unwrap();
                    score += Solver::score_brew(ingredients, order);
                }

                score
            }

            fn score_brew(ingredients: &[i32; INGREDIENT_TIER_COUNT], order: &Order) -> f32 {
                let mut score = 0.0;

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
                score
            }
        }
    }

    /// Models Module
    pub mod models {
        use std::any::Any;
        use std::collections::HashSet;

        use crate::{INGREDIENT_TIER_COUNT, NO_INGREDIENT_CHANGE, REST_ID};

        pub enum ActionType {
            Cast,
            Brew,
            Learn,
            Rest,
        }

        pub trait Action {
            fn get_id(&self) -> &i32;
            fn get_ingredient_change(&self) -> &[i32; INGREDIENT_TIER_COUNT];
            fn get_action_type(&self) -> ActionType;
            fn is_rest(&self) -> bool {
                false
            }
            fn as_any(&self) -> &dyn Any;
        }

        pub struct Rest;

        impl Rest {
            pub fn new() -> Rest {
                Rest {}
            }
        }

        impl Action for Rest {
            fn get_id(&self) -> &i32 {
                &REST_ID
            }

            fn get_ingredient_change(&self) -> &[i32; 4] {
                &NO_INGREDIENT_CHANGE
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Rest
            }

            fn is_rest(&self) -> bool {
                true
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct LearntSpell {
            id: i32,
            ingredient_change: [i32; INGREDIENT_TIER_COUNT],
            //is_repeatable: bool,
        }

        impl LearntSpell {
            pub fn new(
                id: i32,
                ingredient_change: [i32; INGREDIENT_TIER_COUNT],
                /*is_repeatable: bool*/) -> LearntSpell {
                LearntSpell {
                    id,
                    ingredient_change,
                    //is_repeatable,
                }
            }

//            fn is_repeatable(&self) -> bool {
//                self.is_repeatable
//            }
        }

        impl Action for LearntSpell {
            fn get_id(&self) -> &i32 {
                &self.id
            }

            fn get_ingredient_change(&self) -> &[i32; INGREDIENT_TIER_COUNT] {
                &self.ingredient_change
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Cast
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct UnlearntSpell {
            id: i32,
            ingredient_change: [i32; INGREDIENT_TIER_COUNT],
            //is_repeatable: bool,
            read_ahead_tax: i32,
            tax_gain: i32,
        }

        impl UnlearntSpell {
            pub fn new(
                id: i32,
                ingredient_change: [i32; INGREDIENT_TIER_COUNT],
                //is_repeatable: bool,
                read_ahead_tax: i32,
                tax_gain: i32) -> UnlearntSpell {
                UnlearntSpell {
                    id,
                    ingredient_change,
                    //is_repeatable,
                    read_ahead_tax,
                    tax_gain,
                }
            }

            pub fn get_read_ahead_tax(&self) -> i32 {
                self.read_ahead_tax
            }

            pub fn get_tax_gain(&self) -> i32 {
                self.tax_gain
            }

//            fn is_repeatable(&self) -> bool {
//                self.is_repeatable
//            }
        }

        impl Action for UnlearntSpell {
            fn get_id(&self) -> &i32 {
                &self.id
            }

            fn get_ingredient_change(&self) -> &[i32; INGREDIENT_TIER_COUNT] {
                &self.ingredient_change
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Learn
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct Order {
            id: i32,
            ingredient_change: [i32; INGREDIENT_TIER_COUNT],
            price: i32,
        }

        impl Order {
            pub fn new(id: i32, price: i32, ingredient_change: [i32; INGREDIENT_TIER_COUNT]) -> Order {
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

            fn get_ingredient_change(&self) -> &[i32; INGREDIENT_TIER_COUNT] {
                &self.ingredient_change
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Brew
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct State {
            my_ingredients: [i32; INGREDIENT_TIER_COUNT],
            my_rupees: i32,
            inactive_orders: HashSet<i32>,
            inactive_spells: HashSet<i32>,
            learnt_spells: HashSet<i32>,
            root_action_id: Option<i32>,
            depth: i32,
        }

        impl State {
            pub fn new(
                my_ingredients: [i32; INGREDIENT_TIER_COUNT],
                my_rupees: i32,
                inactive_orders: HashSet<i32>,
                inactive_spells: HashSet<i32>,
                learnt_spells: HashSet<i32>,
                root_action_id: Option<i32>,
                depth: i32) -> State {
                State {
                    my_ingredients,
                    my_rupees,
                    inactive_orders,
                    inactive_spells,
                    learnt_spells,
                    root_action_id,
                    depth,
                }
            }

            pub fn get_ingredients(&self) -> &[i32; INGREDIENT_TIER_COUNT] {
                &self.my_ingredients
            }

            pub fn get_rupees(&self) -> &i32 {
                &self.my_rupees
            }

            pub fn get_inactive_orders(&self) -> &HashSet<i32> {
                &self.inactive_orders
            }

            pub fn get_inactive_spells(&self) -> &HashSet<i32> {
                &self.inactive_orders
            }

            pub fn get_learnt_spells(&self) -> &HashSet<i32> {
                &self.learnt_spells
            }

            pub fn get_root_action_id(&self) -> &Option<i32> {
                &self.root_action_id
            }

            pub fn get_depth(&self) -> &i32 {
                &self.depth
            }

            pub fn is_action_active(&self, action_id: &i32) -> bool {
                self.learnt_spells.contains(action_id) ||
                    (!self.inactive_spells.contains(action_id) &&
                        !self.inactive_orders.contains(action_id))
            }

            pub fn deactivate_order(&mut self, action_id: &i32) {
                self.inactive_orders.insert(action_id.clone());
            }

            pub fn deactivate_spell(&mut self, action_id: &i32, is_new_learn: bool) {
                if !is_new_learn && self.learnt_spells.contains(action_id) {
                    self.learnt_spells.remove(action_id);
                } else {
                    self.inactive_spells.insert(action_id.clone());
                }
            }
        }
    }
}

fn main() {
    Game::run();
}
