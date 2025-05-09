use rand::{Rng, rng};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Instance {
    customers: Vec<Customer>,
    vehicle_count: usize,
    vehicle_capacity: usize,
}

#[derive(Debug, Clone, Copy)]
struct Customer {
    id: usize,
    x: f64,
    y: f64,
    demand: usize,
}

#[derive(Debug, Clone)]
struct Route {
    customers: Vec<usize>, // includes depot (0) at start and end
    load: usize,
    cost: f64,
}

type DistanceMatrix = Vec<Vec<f64>>;

fn euclidean(a: &Customer, b: &Customer) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt()
}

fn compute_distance_matrix(customers: &[Customer]) -> DistanceMatrix {
    let n = customers.len();
    let mut dist = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            dist[i][j] = euclidean(&customers[i], &customers[j]);
        }
    }
    dist
}

fn route_cost(route: &[usize], dist: &DistanceMatrix) -> f64 {
    route.windows(2).map(|w| dist[w[0]][w[1]]).sum()
}

fn two_opt(route: &mut Route, dist: &DistanceMatrix) {
    let n = route.customers.len();
    let mut improved = true;
    while improved {
        improved = false;
        for i in 1..n - 2 {
            for j in i + 1..n - 1 {
                let mut new_customers = route.customers.clone();
                new_customers[i..=j].reverse();
                let new_cost = route_cost(&new_customers, dist);
                if new_cost < route.cost {
                    route.customers = new_customers;
                    route.cost = new_cost;
                    improved = true;
                }
            }
        }
    }
}

fn initial_sweep(
    customers: &[Customer],
    dist: &DistanceMatrix,
    vehicle_count: usize,
    capacity: usize,
) -> Vec<Route> {

    // ----------------POLAR SWEEP---------------------------------------------------
    
    // let depot = &customers[0];
    // let mut polar_customers_sorted: Vec<_> = customers
    //     .iter()
    //     .skip(1)
    //     .map(|c| {
    //         let angle = (c.y - depot.y).atan2(c.x - depot.x);
    //         (angle, c)
    //     })
    //     .collect();

    // polar_customers_sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // let index = rng().random_range(0..polar_customers_sorted.len());
    // let mut polar_customers: Vec<_> = polar_customers_sorted[index..].to_vec();
    // polar_customers.extend(&polar_customers_sorted[..index]);

    // // Idea is to get average demand to turn polar sweep into a bin-packing problem sort of thing
    // let mut rem_demand: usize = polar_customers_sorted.iter().map(|(_,c)| c.demand).sum();
    // let mut rem_vehicles = vehicle_count;

    // let mut solution = Vec::with_capacity(vehicle_count);
    // let mut current_route = Route {
    //     customers: vec![0], // Start at depot
    //     load: 0,
    //     cost: 0.0,
    // };

    // for &(_, cust) in &polar_customers {

    //     let current_slot = ((rem_demand as f64/ rem_vehicles as f64).ceil() as usize).min(capacity);

    //     if rem_vehicles > 1 && (current_route.load + cust.demand > capacity || current_route.load > current_slot) {
    //         current_route.customers.push(0); // Return to depot
    //         current_route.cost = route_cost(&current_route.customers, dist);
    //         rem_demand -= current_route.load;
    //         rem_vehicles -= 1;
    //         solution.push(current_route);
            

    //         current_route = Route {
    //             customers: vec![0],
    //             load: 0,
    //             cost: 0.0,
    //         };
    //     }

    //     current_route.customers.push(cust.id);
    //     current_route.load += cust.demand;
    // }


    // // Push final route
    // if current_route.customers.len() > 1 {
    //     current_route.customers.push(0);
    //     current_route.cost = route_cost(&current_route.customers, dist);
    //     solution.push(current_route);
    // }

    // // Pad with empty routes if fewer than vehicle_count
    // while solution.len() < vehicle_count {
    //     solution.push(Route {
    //         customers: vec![0, 0],
    //         load: 0,
    //         cost: 0.0,
    //     });
    // }

    // solution

    // ----------------------------------------------------------------------------------


    // ----------------BIN PACKING------------------------------------------------------

    // Sort the whole thingamabob by demand descending - normal bin packing 
    let mut og_demand_sorted: Vec<_> = customers.iter().skip(1).collect();
    og_demand_sorted.sort_by(|a, b| b.demand.cmp(&a.demand));

    // Introduce randomness by rotating the list (we need some type of randomness to make the existing restarts worthwhile and this shouldn't break anything)
    let index = rng().random_range(0..og_demand_sorted.len());
    let mut demand_sorted: Vec<_> = og_demand_sorted[index..].to_vec();
    demand_sorted.extend(&og_demand_sorted[..index]);

    let mut solution: Vec<Route> = Vec::with_capacity(vehicle_count);

    for &cust in &demand_sorted {

        let mut found = false;
        let mut best_route_idx = 0;
        let mut best_insert_pos = 0;
        let mut best_diff = 0.0;
        
        for (r_idx, route) in solution.iter().enumerate() {

            // Skip a route if it can't be inserted
            if route.load + cust.demand > capacity { 
                continue; 
            }
            
            // try every possible insertion in a route
            for pos in 1..route.customers.len() {
                
                let before = route.customers[pos - 1];
                let after  = route.customers[pos];
                let curr_diff = dist[before][cust.id]
                          + dist[cust.id][after]
                          - dist[before][after];
                if !found || curr_diff < best_diff {
                    found = true;
                    best_diff = curr_diff;
                    best_route_idx = r_idx;
                    best_insert_pos = pos;
                }

            }
        }
        
        if found {
            let r = &mut solution[best_route_idx];
            r.customers.insert(best_insert_pos, cust.id);
            r.load += cust.demand;
            r.cost += best_diff;
        } else {
            solution.push(Route {
                customers: vec![0, cust.id, 0],
                load: cust.demand,
                cost: dist[0][cust.id] * 2.0,
            });
        }
    }

    // Pad with empty routes if fewer than vehicle_count
    while solution.len() < vehicle_count {
        solution.push(Route {
            customers: vec![0, 0],
            load: 0,
            cost: 0.0,
        });
    }

    solution

    // ----------------------------------------------------------------------------------
}

fn initial(
    customers: &[Customer],
    dist: &DistanceMatrix,
    vehicle_count: usize,
    capacity: usize,
) -> Vec<Route> {
    initial_sweep(customers, dist, vehicle_count, capacity)
}

fn total_cost(solution: &[Route]) -> f64 {
    solution.iter().map(|r| r.cost).sum()
}

fn perturb(solution: &mut [Route], customers: &[Customer], dist: &DistanceMatrix, capacity: usize) {
    let mut rng = rng();
    let (v1, v2) = {
        let mut i = rng.random_range(0..solution.len());
        let mut j = rng.random_range(0..solution.len());
        while i == j || solution[i].customers.len() <= 2 || solution[j].customers.len() <= 2 {
            i = rng.random_range(0..solution.len());
            j = rng.random_range(0..solution.len());
        }
        (i, j)
    };

    let i = rng.random_range(1..solution[v1].customers.len() - 1);
    let j = rng.random_range(1..solution[v2].customers.len() - 1);
    let c1 = solution[v1].customers[i];
    let c2 = solution[v2].customers[j];

    let load1 = solution[v1].load - customers[c1].demand + customers[c2].demand;
    let load2 = solution[v2].load - customers[c2].demand + customers[c1].demand;

    if load1 <= capacity && load2 <= capacity {
        solution[v1].customers[i] = c2;
        solution[v2].customers[j] = c1;
        solution[v1].load = load1;
        solution[v2].load = load2;
        solution[v1].cost = route_cost(&solution[v1].customers, dist);
        solution[v2].cost = route_cost(&solution[v2].customers, dist);
    }
}

fn accept(delta: f64, temp: f64) -> bool {
    if delta < 0.0 {
        true
    } else {
        let prob = (-delta / temp).exp();
        rng().random::<f64>() < prob
    }
}

// fn solve_cvrp(
// customers: &[Customer],
// vehicle_count: usize,
// capacity: usize,
// time_limit: Duration,
// ) -> Vec<Route> {
// let dist = compute_distance_matrix(customers);
// let mut best = initial(customers, &dist, vehicle_count, capacity);
// let mut current = best.clone();
// let mut best_cost = total_cost(&best);
// let mut temp = 10000.0;

// let start = Instant::now();

// let mut stagnation_counter = 0;
// let stagnation_limit = 10_000;

// while start.elapsed() < time_limit {
// let mut candidate = current.clone();
// perturb(&mut candidate, customers, &dist, capacity);
// for route in &mut candidate {
// two_opt(route, &dist);
// }

// let candidate_cost = total_cost(&candidate);
// if accept(candidate_cost - total_cost(&current), temp) {
// current = candidate;
// if candidate_cost < best_cost {
// best = current.clone();
// best_cost = candidate_cost;
// stagnation_counter = 0; // reset on improvement
// } else {
// stagnation_counter += 1;
// }
// } else {
// stagnation_counter += 1;
// }

// if stagnation_counter >= stagnation_limit {
// println!("Terminating early due to stagnation.");
// break;
// }

// temp *= 0.995;
// }

// best
// }

fn solve_cvrp_with_restarts(
    customers: &[Customer],
    vehicle_count: usize,
    capacity: usize,
    total_time_limit: Duration,
) -> Vec<Route> {
    let dist = compute_distance_matrix(customers);
    let mut best_overall = Vec::new();
    let mut best_cost = f64::INFINITY;

    // let mut stagnation_counter = 0;
    // let stagnation_limit = 100;

    let start = Instant::now();

    while start.elapsed() < total_time_limit {
        let elapsed = start.elapsed();
        let remaining_time = if total_time_limit > elapsed {
            total_time_limit - elapsed
        } else {
            break;
        };

        // Generate randomized initial solution
        let initial = initial(customers, &dist, vehicle_count, capacity);

        // Simulated annealing on this initial solution
        let solution = solve_cvrp_sa(customers, &dist, initial, capacity, remaining_time);

        let cost = total_cost(&solution);
        if cost < best_cost {
            best_cost = cost;
            best_overall = solution;
            // stagnation_counter = 0;
        } else {
            // stagnation_counter += 1;
        }

        // if stagnation_counter >= stagnation_limit {
        // break;
        // }
    }

    best_overall
}

fn solve_cvrp_sa(
    customers: &[Customer],
    dist: &DistanceMatrix,
    mut current: Vec<Route>,
    capacity: usize,
    time_limit: Duration,
) -> Vec<Route> {
    let mut best = current.clone();
    let mut best_cost = total_cost(&best);
    let mut temp = 1000.0;
    let mut stagnation_counter = 0;
    let stagnation_limit = 10_000;
    let start = Instant::now();

    while start.elapsed() < time_limit {
        let mut candidate = current.clone();
        perturb(&mut candidate, customers, dist, capacity);
        for route in &mut candidate {
            two_opt(route, dist);
        }

        let candidate_cost = total_cost(&candidate);
        if accept(candidate_cost - total_cost(&current), temp) {
            current = candidate;
            if candidate_cost < best_cost {
                best = current.clone();
                best_cost = candidate_cost;
                stagnation_counter = 0;
            } else {
                stagnation_counter += 1;
            }
        } else {
            stagnation_counter += 1;
        }

        if stagnation_counter >= stagnation_limit {
            break;
        }

        temp *= 0.995;
    }

    best
}

fn parse(filename: &str) -> Instance {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error parsing {}: {}", filename, e);
            process::exit(-1);
        }
    };

    let reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines().map_while(Result::ok).collect();

    let first_line: Vec<_> = lines[0].split_whitespace().collect();
    let num_customers = first_line[0].parse::<usize>().unwrap();
    let vehicle_count = first_line[1].parse::<usize>().unwrap();
    let vehicle_capacity = first_line[2].parse::<usize>().unwrap();

    println!("Number of customers: {}", num_customers);
    println!("Number of vehicles: {}", vehicle_count);
    println!("Vehicle capacity: {}", vehicle_capacity);

    let mut customers = Vec::new();
    for (id, line) in lines[1..].iter().enumerate() {
        println!("Processing line: {}", line);
        let parts: Vec<_> = line.split_whitespace().collect();
        let demand = parts[0].parse::<usize>().unwrap();
        let x = parts[1].parse::<f64>().unwrap();
        let y = parts[2].parse::<f64>().unwrap();

        customers.push(Customer { id, demand, x, y });
    }

    for customer in &customers {
        println!("{} {} {}", customer.demand, customer.x, customer.y);
    }

    Instance {
        customers,
        vehicle_count,
        vehicle_capacity,
    }
}

fn output(solution: &Vec<Route>, filename: &str) {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error creating {}: {}", filename, e);
            process::exit(-1);
        }
    };

    let mut output = String::new();
    output.push_str(&format!("{} 0\n", total_cost(solution)));

    println!("FINAL");
    for route in solution {
        println!("{}", route.load);

        output.push_str(
            &route
                .customers
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
        output.push('\n');
    }

    file.write_all(output.as_bytes()).unwrap();
}

fn main() {
    let filename = std::env::args().nth(1).unwrap();

    let now = Instant::now();
    let instance = parse(&filename);
    let solution = solve_cvrp_with_restarts(
        &instance.customers,
        instance.vehicle_count,
        instance.vehicle_capacity,
        Duration::from_secs(5),
    );
    let time = now.elapsed();

    output(&solution, &format!("{}.sol", filename));

    let result = total_cost(&solution);
    let sol = solution
        .iter()
        .map(|route| {
            route
                .customers
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" ");

    print!(
        r#"{{"Instance": "{instance}", "Time": "{time:.2}", "Result": "{result}", "Solution": "{sol}"}}"#,
        instance = filename,
        time = time.as_secs_f64(),
        result = result,
        sol = sol,
    );
}
