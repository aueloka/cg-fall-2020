extern crate rand;

use crate::solution::execution::{Game, Strategy};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

const TIMEOUT: u128 = 50;
const MAX_DEPTH: i32 = 3;
const NULL_ACTION_ID: (i32, i32) = (-1, 0);
const INGREDIENT_TIER_COUNT: usize = 4;
const REST_ID: (i32, i32) = (-50, 0);
const MAX_INGREDIENT_COUNT: i32 = 10;
const NO_INGREDIENT_CHANGE: [i32; INGREDIENT_TIER_COUNT] = [0; 4];
const MAX_REPEAT_COUNT: i32 = 2;
const STRATEGY: Strategy = Strategy::BreadthFirstSearch;

mod solution {
    pub mod execution {
        use std::collections::HashSet;
        use std::io;

        use crate::{INGREDIENT_TIER_COUNT, MAX_REPEAT_COUNT, STRATEGY};
        use crate::solution::models::{ActionType, Order, UnlearntSpell};
        use crate::solution::models::LearntSpell;
        use crate::solution::models::State;
        use crate::solution::runtime::{ActionsRepository, BestFirstSolutionFinder, BreadthFirstSolutionFinder, DefaultStateEvaluator, RandomStateEvaluator, SolutionFinder};

        pub enum Strategy {
            BestFirstSearch,
            BreadthFirstSearch,
        }

        pub struct Game;

        impl Game {
            pub fn run() {
                // game loop
                loop {
                    let (repo, my_inactive_spells) = Game::read_actions();
                    let state_evaluator =
                        //Box::new(RandomStateEvaluator::new());
                        Box::new(DefaultStateEvaluator::new());

                    let solution_finder: Box<dyn SolutionFinder> = match STRATEGY {
                        Strategy::BreadthFirstSearch => Box::new(BreadthFirstSolutionFinder::new(
                            state_evaluator)),
                        Strategy::BestFirstSearch => Box::new(BestFirstSolutionFinder::new(
                            state_evaluator))
                    };

                    let state = Game::read_state(my_inactive_spells);
                    let best_action = solution_finder.search(state, &repo);

                    // in the first league: BREW <id> | WAIT; later: BREW <id> | CAST <id> [<times>] | LEARN <id> | REST | WAIT
                    if let Some(action) = repo.get_action(&best_action) {
                        match action.get_action_type() {
                            ActionType::Cast => println!("CAST {} {}", &best_action.0, &best_action.1 /*We only cast once for now. TODO: Evaluate how to choose the best times*/),
                            ActionType::Brew => println!("BREW {}", &best_action.0),
                            ActionType::Learn => println!("LEARN {}", &best_action.0),
                            ActionType::Rest => println!("REST"),
                        }

                        continue;
                    }

                    eprintln!("No valid action returned. {:?}", &best_action);

                    //No action was found.
                    println!("WAIT");
                }
            }

            fn read_actions() -> (Box<ActionsRepository>, HashSet<i32>) {
                let mut repo = Box::new(ActionsRepository::new());
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
                    let action_id: i32 = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
                    let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW
                    let action_type: &str = &action_type[..];
                    let delta_0: i32 = parse_input!(inputs[2], i32); // tier-0 ingredient change
                    let delta_1: i32 = parse_input!(inputs[3], i32); // tier-1 ingredient change
                    let delta_2: i32 = parse_input!(inputs[4], i32); // tier-2 ingredient change
                    let delta_3: i32 = parse_input!(inputs[5], i32); // tier-3 ingredient change
                    let price: i32 = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
                    let tome_index: i32 = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax
                    let tax_count: i32 = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell
                    let castable: i32 = parse_input!(inputs[9], i32); // in the first league: always 0; later: 1 if this is a castable player spell
                    let repeatable: i32 = parse_input!(inputs[10], i32); // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

                    match action_type {
                        "OPPONENT_CAST" => {}
                        "CAST" => {
                            if repeatable == 0 {
                                let new_action_id = (action_id, 1);

                                repo.add_learnt_spell(new_action_id, Box::new(LearntSpell::new(
                                    [delta_0, delta_1, delta_2, delta_3],
                                    //repeatable == 1,
                                )));

                                if castable != 1 {
                                    my_inactive_spells.insert(action_id);
                                }
                            } else {
                                for i in 1..MAX_REPEAT_COUNT + 1 {
                                    let repeatable_action_id = (action_id, i);

                                    repo.add_learnt_spell(repeatable_action_id, Box::new(LearntSpell::new(
                                        [delta_0 * i, delta_1 * i, delta_2 * i, delta_3 * i],
                                        //repeatable == 1,
                                    )));

                                    if castable != 1 {
                                        my_inactive_spells.insert(action_id);
                                    }
                                }
                            }
                        }
                        "LEARN" => {
                            if repeatable == 0 {
                                let new_action_id = (action_id, 1);

                                repo.add_unlearnt_spell(new_action_id, Box::new(UnlearntSpell::new(
                                    [delta_0, delta_1, delta_2, delta_3],
                                    //repeatable == 1,
                                    tome_index,
                                    tax_count,
                                )));
                            } else {
                                for i in 1..MAX_REPEAT_COUNT + 1 {
                                    let repeatable_action_id = (action_id, i);

                                    repo.add_unlearnt_spell(repeatable_action_id, Box::new(UnlearntSpell::new(
                                        [delta_0 * i, delta_1 * i, delta_2 * i, delta_3 * i],
                                        //repeatable == 1,
                                        tome_index,
                                        tax_count,
                                    )));
                                }
                            }
                        }
                        "BREW" => {
                            repo.add_order((action_id, 0), Box::new(Order::new(
                                price,
                                [delta_0, delta_1, delta_2, delta_3])));
                        }
                        _ => eprintln!("Unrecognized action type: {}", action_type),
                    }
                }

                (repo, my_inactive_spells)
            }

            fn read_state(my_inactive_spells: HashSet<i32>) -> Box<State> {
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

                Box::new(State::new(
                    my_ingredients,
                    my_rupees,
                    HashSet::new(),
                    my_inactive_spells,
                    HashSet::new(),
                    None,
                    0))
            }
        }
    }

    /// Runtime Module
    pub mod runtime {
        use rand::{random, Rng, thread_rng};
        use std::cmp::min;
        use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
        use std::time::Instant;

        use crate::{INGREDIENT_TIER_COUNT, MAX_DEPTH, MAX_INGREDIENT_COUNT, NULL_ACTION_ID, REST_ID, TIMEOUT};
        use crate::solution::models::{Action, ActionType, LearntSpell, Order, Rest, State, UnlearntSpell};

        pub struct ActionsRepository {
            actions: HashMap<(i32, i32), Box<dyn Action>>,
            orders: Vec<(i32, i32)>,
            learnt_spells: Vec<(i32, i32)>,
        }

        impl ActionsRepository {
            pub fn new() -> ActionsRepository {
                ActionsRepository {
                    actions: HashMap::new(),
                    orders: Vec::new(),
                    learnt_spells: Vec::new(),
                }
            }

            pub fn add_order(&mut self, id: (i32, i32), order: Box<Order>) {
                self.actions.insert(id, order);
                self.orders.push(id);
            }

            pub fn add_learnt_spell(&mut self, id: (i32, i32), spell: Box<LearntSpell>) {
                self.actions.insert(id, spell);
                self.learnt_spells.push(id);
            }

            pub fn add_unlearnt_spell(&mut self, id: (i32, i32), spell: Box<UnlearntSpell>) {
                self.actions.insert(id, spell);
            }

            pub fn add_rest(&mut self) {
                self.actions.insert(REST_ID, Box::new(Rest::new()));
            }

            pub fn get_action(&self, id: &(i32, i32)) -> Option<&Box<dyn Action>> {
                self.actions.get(id)
            }

            pub fn get_action_ids(&self) -> Vec<&(i32, i32)> {
                self.actions.keys().collect()
            }

            pub fn get_order_ids(&self) -> &Vec<(i32, i32)> {
                &self.orders
            }

            pub fn get_learnt_spell_ids(&self) -> &Vec<(i32, i32)> {
                &self.learnt_spells
            }
        }

        /// Executes actions
        pub struct ActionExecutor;

        impl ActionExecutor {
            pub fn execute(repo: &Box<ActionsRepository>, state: &State, action_id: &(i32, i32)) -> Option<Box<State>> {
                let action: Option<&Box<dyn Action>> = repo.get_action(action_id);

                if action.is_none() {
                    return None;
                }

                let action = action.unwrap();
                let root_action_id = state.get_root_action_id().unwrap_or(*action_id);
//
//                if true {
//                    let mut rng = thread_rng();
//
//                    return Some(Box::new(
//                        State::new(
//                            [0;4],
//                            0,
//                            HashSet::new(),
//                            HashSet::new(),
//                            HashSet::new(),
//                            Some(root_action_id),
//                            state.get_depth() + 1,
//                        )
//                    ))
//                }

                let current_ingredients = state.get_ingredients();

                if action.is_rest() {
                    return Some(Box::new(State::new(
                        [current_ingredients[0], current_ingredients[1], current_ingredients[2], current_ingredients[3]],
                        *state.get_rupees(),
                        state.get_inactive_orders().clone(),
                        HashSet::new(), //Reset inactive spells
                        state.get_learnt_spells().clone(),
                        Some(root_action_id),
                        state.get_depth() + 1)));
                }

                let mut default_ingredient_change = [0; INGREDIENT_TIER_COUNT];

                let ingredient_change = match action.get_action_type() {
                    ActionType::Learn => {
                        if state.get_learnt_spells().contains(&action_id.0) {
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

                let mut new_rupees = *state.get_rupees();
                let mut learnt_spells = state.get_learnt_spells().clone();
                let mut is_new_learn = false;

                //Action specific customization
                match action.get_action_type() {
                    ActionType::Brew => {
                        //eprintln!("Evaluating brew");
                        let order: &Order = action.as_any().downcast_ref::<Order>().unwrap();
                        new_rupees += order.get_price();
                    }
                    ActionType::Learn => {
                        if !state.get_learnt_spells().contains(&action_id.0) {
                            //eprintln!("Evaluating learn");
                            let unlearnt_spell: &UnlearntSpell = action.as_any().downcast_ref::<UnlearntSpell>().unwrap();
                            new_ingredients[0] += min(MAX_INGREDIENT_COUNT - total_ingredients, unlearnt_spell.get_tax_gain());
                            learnt_spells.insert(action_id.0);
                            is_new_learn = true;
                        }
                    }
                    _ => {}
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
                    ActionType::Rest => {}
                    _ => new_state.deactivate_spell(action_id, is_new_learn),
                }

                //eprintln!("New state: {:#?}", new_state);
                Some(Box::new(new_state))
            }
        }

        pub struct StateExpander;

        impl StateExpander {
            pub fn get_children(state: &Box<State>, repo: &Box<ActionsRepository>, time: &Instant, state_evaluator: &Box<dyn StateEvaluator>) -> Vec<Box<State>> {
                let mut new_states: Vec<Box<State>> = Vec::new();

//                let mut seen: HashSet<i32> = HashSet::new();

                for action in repo.get_action_ids() {
                    if !state.is_action_active(&action) {
                        continue;
                    }

//                    if action.1 != 0 && repo.get_action(action).unwrap().is_learn() && seen.contains(&action.0) {
//                        continue;
//                    } else {
//                        seen.insert(action.0);
//                    }

                    if time.elapsed().as_millis() >= TIMEOUT - 2 {
                        break;
                    }

                    let new_state = ActionExecutor::execute(repo, state, action);

                    if new_state.is_some() {
                        let mut new_state = new_state.unwrap();
                        new_state.set_score(state_evaluator.evaluate(state, repo));
                        new_states.push(new_state);
                    }
                }

                //eprintln!("Branches for state at depth: {}, {}", state.get_depth(), new_states.len());
                new_states
            }
        }

        pub trait StateEvaluator {
            fn evaluate(&self, state: &Box<State>, repo: &Box<ActionsRepository>) -> f32;
        }

        pub struct DefaultStateEvaluator;

        impl DefaultStateEvaluator {
            pub fn new() -> DefaultStateEvaluator {
                DefaultStateEvaluator {}
            }
        }

        impl StateEvaluator for DefaultStateEvaluator {
            fn evaluate(&self, state: &Box<State>, repo: &Box<ActionsRepository>) -> f32 {
                let mut score = 0.0;

                let ingredients = state.get_ingredients();

//                if repo.get_learnt_spell_ids().len() < 15 {
//                    score += state.get_learnt_spells().len() as f32 * 200.0;
//                    score += ingredients[0] as f32 * 10.0;
//                    score += ingredients[1] as f32 * 20.0;
//                    score += ingredients[2] as f32 * 30.0;
//                    score += ingredients[3] as f32 * 40.0;
//                }

//                for spell_id in repo.get_learnt_spell_ids() {
//                    let spell = repo.get_action(spell_id).unwrap();
//
//                    let delta = spell.get_ingredient_change();
//
//                    score += delta[0] as f32 * 0.5;
//                    score += delta[1] as f32 * 1.0;
//                    score += delta[2] as f32 * 2.0;
//                    score += delta[3] as f32 * 3.0;
//                }


                //We get rewarded for these as well.
                //score += (ingredients[1] + ingredients[2] + ingredients[3]) as f32 * 0.0;

                for order_id in repo.get_order_ids() {
                    let action = repo.get_action(order_id).unwrap();
                    let order: &Order = action.as_any().downcast_ref::<Order>().unwrap();
                    let order_price = order.get_price().clone() as f32;
                    let order_requirement = order.get_ingredient_change();

                    if state.get_inactive_orders().contains(&order_id.0) {
                        // Having rupees is the best place to be :).
                        score += order_price * 100.0;
                        score += order_requirement[0] as f32 * -0.5;
                        score += order_requirement[1] as f32 * -1.0;
                        score += order_requirement[2] as f32 * -2.0;
                        score += order_requirement[3] as f32 * -3.0;

                        //if debug { eprintln!("Added reward {} to score for order {}", completion_reward, order_id); }
                        continue;
                    }


                    //eprintln!("Adding score for order: {:#?}", order);

                    let order_distance = min(ingredients[0] + order_requirement[0], 0) +
                        min(ingredients[1] + order_requirement[1], 0) +
                        min(ingredients[2] + order_requirement[2], 0) +
                        min(ingredients[3] + order_requirement[3], 0);

                    //eprintln!("Order distance: {}", order_distance);

                    if order_distance >= 0 {
                        // We're rewarding being able to brew with half the price of the order. i.e if we can brew 2 orders. go with the one that pays more.

                        //if debug { eprintln!("Added distance reward {} to score for order {}", order_price * 0.3, order_id); }
                        score += order_price * 0.3;
                    }

                    // We're punishing not being able to produce orders with half the distance.
                    // We can multiply by the order price to have the price of the order influence what spells to cast.
                    // i.e if we can cast 2 spells that will open up 2 orders, open the one that pays more

                    //if debug && order_distance != 0 { eprintln!("Added distance punishment {} to score for order {}", order_distance as f32 * order_price * 0.3, order_id); }
                    score += order_distance as f32 * order_price * 0.1;
                }

                score
            }
        }

        pub struct RandomStateEvaluator;

        impl RandomStateEvaluator {
            pub fn new() -> RandomStateEvaluator {
                RandomStateEvaluator {}
            }
        }

        impl StateEvaluator for RandomStateEvaluator {
            fn evaluate(&self, state: &Box<State>, repo: &Box<ActionsRepository>) -> f32 {
                //rand::random()
                *state.get_rupees() as f32
            }
        }

        pub trait SolutionFinder {
            fn search(&self, state: Box<State>, repo: &Box<ActionsRepository>) -> (i32, i32);
        }

        pub struct BreadthFirstSolutionFinder {
            state_evaluator: Box<dyn StateEvaluator>,
        }

        impl BreadthFirstSolutionFinder {
            pub fn new(state_evaluator: Box<dyn StateEvaluator>) -> BreadthFirstSolutionFinder {
                BreadthFirstSolutionFinder {
                    state_evaluator,
                }
            }
        }

        impl SolutionFinder for BreadthFirstSolutionFinder {
            fn search(&self, state: Box<State>, repo: &Box<ActionsRepository>) -> (i32, i32) {
                let time = Instant::now();
                eprintln!("Ingredients: {:?}", &state.get_ingredients());

                let mut queue: VecDeque<Box<State>> = VecDeque::new();
                queue.push_back(state);

                let mut score_map: HashMap<(i32, i32), (i32, f32)> = HashMap::new(); //<root_id, <depth, score>>

                let mut node_count = 0;

                while let Some(current_state) = queue.pop_front() {
                    if time.elapsed().as_millis() >= TIMEOUT - 1 {
                        eprintln!("TIMEOUT. Depth: {}", &current_state.get_depth());
                        break;
                    }

                    let root_action_id = current_state.get_root_action_id().unwrap_or(NULL_ACTION_ID);
                    let score = current_state.get_score();//self.state_evaluator.evaluate(&current_state, repo);

                    //eprintln!("Exploring state with root action {} at depth {} with score {}", root_action_id.0, current_state.get_depth(), score);

                    node_count += 1;

                    if root_action_id != NULL_ACTION_ID {
                        if let Some(pair) = score_map.get_mut(&root_action_id) {
                            if pair.0 > *current_state.get_depth() || *score > pair.1 {
                                score_map.insert(root_action_id, (*current_state.get_depth(), *score));
                            }
                        } else {
                            score_map.insert(root_action_id, (*current_state.get_depth(), *score));
                        }
                    }

                    if current_state.get_depth() >= &MAX_DEPTH {
                        //eprintln!("Max depth reached.");
                        continue;
                    }

                    for child in StateExpander::get_children(&current_state, repo, &time, &self.state_evaluator) {
                        queue.push_back(child)
                    }
                }

                eprintln!("Evaluated {} nodes.", &node_count);
                let mut best: ((i32, i32), f32) = (NULL_ACTION_ID, std::f32::MIN); //best.1

                for key in score_map.keys() {
                    let value = score_map.get(key).unwrap();
                    //eprintln!("Action: {}, Score: {}. Depth: {}", key, &value.1, &value.0);

                    if value.1 > best.1 {
                        //eprintln!("New best: ({}, {})", key, &value.1);
                        best = (*key, value.1);
                    }
                }

                best.0
            }
        }

        pub struct BestFirstSolutionFinder {
            state_evaluator: Box<dyn StateEvaluator>,
        }

        impl BestFirstSolutionFinder {
            pub fn new(state_evaluator: Box<dyn StateEvaluator>) -> BestFirstSolutionFinder {
                BestFirstSolutionFinder {
                    state_evaluator,
                }
            }
        }

        impl SolutionFinder for BestFirstSolutionFinder {
            fn search(&self, state: Box<State>, repo: &Box<ActionsRepository>) -> (i32, i32) {
                let time = Instant::now();
                eprintln!("Ingredients: {:?}", &state.get_ingredients());

                let mut queue: BinaryHeap<Box<State>> = BinaryHeap::new();
                queue.push(state);

                let mut score_map: HashMap<(i32, i32), (i32, f32)> = HashMap::new(); //<root_id, <depth, score>>

                let mut node_count = 0;

                while let Some(current_state) = queue.pop() {
                    if time.elapsed().as_millis() >= TIMEOUT - 1 {
                        eprintln!("TIMEOUT. Depth: {}", &current_state.get_depth());
                        break;
                    }

                    let root_action_id = current_state.get_root_action_id().unwrap_or(NULL_ACTION_ID);
                    let score = current_state.get_score();//self.state_evaluator.evaluate(&current_state, repo);

                    //eprintln!("Exploring state with root action {} at depth {} with score {}", root_action_id.0, current_state.get_depth(), score);

                    node_count += 1;

                    if root_action_id != NULL_ACTION_ID {
                        if let Some(pair) = score_map.get_mut(&root_action_id) {
                            if pair.0 > *current_state.get_depth() || *score > pair.1 {
                                score_map.insert(root_action_id, (*current_state.get_depth(), *score));
                            }
                        } else {
                            score_map.insert(root_action_id, (*current_state.get_depth(), *score));
                        }
                    }

                    if current_state.get_depth() >= &MAX_DEPTH {
                        //eprintln!("Max depth reached.");
                        continue;
                    }

                    for child in StateExpander::get_children(&current_state, repo, &time, &self.state_evaluator) {
                        queue.push(child)
                    }
                }

                eprintln!("Evaluated {} nodes.", &node_count);
                let mut best: ((i32, i32), f32) = (NULL_ACTION_ID, std::f32::MIN);

                for key in score_map.keys() {
                    let value = score_map.get(key).unwrap();
                    //eprintln!("Action: {}, Score: {}. Depth: {}", key, &value.1, &value.0);

                    if value.1 > best.1 {
                        eprintln!("New best: ({}, {}). Depth: {}", key.0, &value.1, value.0);
                        best = (*key, value.1);
                    }
                }

                best.0
            }
        }
    }

    /// Models Module
    pub mod models {
        use std::any::Any;
        use std::cmp::{Ordering, PartialOrd};
        use std::collections::HashSet;
        use std::hash::{Hash, Hasher};

        use crate::{INGREDIENT_TIER_COUNT, NO_INGREDIENT_CHANGE};

        pub enum ActionType {
            Cast,
            Brew,
            Learn,
            Rest,
        }

        pub trait Action {
            fn get_ingredient_change(&self) -> &[i32; INGREDIENT_TIER_COUNT];
            fn get_action_type(&self) -> ActionType;

            fn is_rest(&self) -> bool {
                false
            }

            fn is_learn(&self) -> bool {
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
            ingredient_change: [i32; INGREDIENT_TIER_COUNT],
        }

        impl LearntSpell {
            pub fn new(
                ingredient_change: [i32; INGREDIENT_TIER_COUNT]) -> LearntSpell {
                LearntSpell {
                    ingredient_change,
                }
            }
        }

        impl Action for LearntSpell {
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
            ingredient_change: [i32; INGREDIENT_TIER_COUNT],
            //is_repeatable: bool,
            read_ahead_tax: i32,
            tax_gain: i32,
        }

        impl UnlearntSpell {
            pub fn new(
                ingredient_change: [i32; INGREDIENT_TIER_COUNT],
                //is_repeatable: bool,
                read_ahead_tax: i32,
                tax_gain: i32) -> UnlearntSpell {
                UnlearntSpell {
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
            fn get_ingredient_change(&self) -> &[i32; INGREDIENT_TIER_COUNT] {
                &self.ingredient_change
            }

            fn get_action_type(&self) -> ActionType {
                ActionType::Learn
            }

            fn is_learn(&self) -> bool {
                true
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct Order {
            ingredient_change: [i32; INGREDIENT_TIER_COUNT],
            price: i32,
        }

        impl Order {
            pub fn new(price: i32, ingredient_change: [i32; INGREDIENT_TIER_COUNT]) -> Order {
                Order {
                    price,
                    ingredient_change,
                }
            }

            pub fn get_price(&self) -> &i32 {
                &self.price
            }
        }

        impl Action for Order {
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

        #[derive(PartialEq)]
        pub struct State {
            my_ingredients: [i32; INGREDIENT_TIER_COUNT],
            my_rupees: i32,
            inactive_orders: HashSet<i32>,
            inactive_spells: HashSet<i32>,
            learnt_spells: HashSet<i32>,
            root_action_id: Option<(i32, i32)>,
            depth: i32,
            score: f32,
        }

        impl Hash for State {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.my_rupees.hash(state);
                self.my_ingredients.hash(state);

                for inactive_order in &self.inactive_orders {
                    inactive_order.hash(state);
                }

                for inactive_spell in &self.inactive_spells {
                    inactive_spell.hash(state);
                }

                for learnt_spell in &self.learnt_spells {
                    learnt_spell.hash(state);
                }
            }
        }

        impl Eq for State {}

        impl Ord for State {
            fn cmp(&self, other: &State) -> Ordering {
                self.score.partial_cmp(&other.score).unwrap()
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &State) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl State {
            pub fn new(
                my_ingredients: [i32; INGREDIENT_TIER_COUNT],
                my_rupees: i32,
                inactive_orders: HashSet<i32>,
                inactive_spells: HashSet<i32>,
                learnt_spells: HashSet<i32>,
                root_action_id: Option<(i32, i32)>,
                depth: i32) -> State {
                State {
                    my_ingredients,
                    my_rupees,
                    inactive_orders,
                    inactive_spells,
                    learnt_spells,
                    root_action_id,
                    depth,
                    score: 0.0,
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

            pub fn get_root_action_id(&self) -> &Option<(i32, i32)> {
                &self.root_action_id
            }

            pub fn get_depth(&self) -> &i32 {
                &self.depth
            }

            pub fn get_score(&self) -> &f32 {
                &self.score
            }

            pub fn set_score(&mut self, score: f32) {
                self.score = score;
            }

            pub fn is_action_active(&self, action_id: &(i32, i32)) -> bool {
                self.learnt_spells.contains(&action_id.0) ||
                    (!self.inactive_spells.contains(&action_id.0) &&
                        !self.inactive_orders.contains(&action_id.0))
            }

            pub fn deactivate_order(&mut self, action_id: &(i32, i32)) {
                self.inactive_orders.insert(action_id.0);
            }

            pub fn deactivate_spell(&mut self, action_id: &(i32, i32), is_new_learn: bool) {
                if !is_new_learn && self.learnt_spells.contains(&action_id.0) {
                    self.learnt_spells.remove(&action_id.0);
                } else {
                    self.inactive_spells.insert(action_id.0);
                }
            }
        }
    }
}

fn main() {
    Game::run();
}
