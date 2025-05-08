import sys
import random

from docplex.mp.model import Model
import numpy  as np

class VRPInstance:
    def __init__(self, file_name):
        # VRP Input Parameters
        self.num_customers = 0  # the number of customers
        self.num_vehicles = 0   # the number of vehicles
        self.vehicle_capacity = 0  # the capacity of the vehicles
        self.demand_of_customer = []  # the demand of each customer
        self.x_coord_of_customer = []  # the x coordinate of each customer
        self.y_coord_of_customer = []  # the y coordinate of each customer

        try:
            with open(file_name, 'r') as file:
                lines = file.readlines()
        except FileNotFoundError as e:
            print(f"Error: in VRPInstance() {file_name}\n{e}")
            sys.exit(-1)

        first_line = lines[0].split()
        self.num_customers = int(first_line[0])
        self.num_vehicles = int(first_line[1])
        self.vehicle_capacity = int(first_line[2])

        print(f"Number of customers: {self.num_customers}")
        print(f"Number of vehicles: {self.num_vehicles}")
        print(f"Vehicle capacity: {self.vehicle_capacity}")

        # Read the remaining lines for customer data
        for line in lines[1:]:
            print(f"Processing line: {line.strip()}")
            data = line.split()
            self.demand_of_customer.append(int(data[0]))
            self.x_coord_of_customer.append(float(data[1]))
            self.y_coord_of_customer.append(float(data[2]))

        # Print the customer data
        for i in range(self.num_customers):
            print(f"{self.demand_of_customer[i]} {self.x_coord_of_customer[i]} {self.y_coord_of_customer[i]}")


class IPSolver:
    def __init__(self, instance: VRPInstance):
        self.instance = instance
        self.model = Model()

        # Step 1 : Add variables to model - edges of the graph for each vehicle
        self.edges_for_each_vehicle = np.empty((self.instance.num_vehicles, self.instance.num_customers, self.instance.num_customers), dtype=object)
        
        for v in range(self.instance.num_vehicles):
            for i in range(self.instance.num_customers):
                for j in range(self.instance.num_customers):
                    if i != j:
                        self.edges_for_each_vehicle[v, i, j] = self.model.binary_var(name=f"x_{v}_{i}_{j}")

        # print("self.edges_for_each_vehicle", self.edges_for_each_vehicle)
        # print("xcoord_of_customer", self.instance.x_coord_of_customer)
        # print("ycoord_of_customer", self.instance.y_coord_of_customer)

        # MTZ Formulation Variables
        self.u = np.empty((self.instance.num_customers), dtype=object)

        for i in range(1, self.instance.num_customers):
            self.u[i] = self.model.continuous_var(name=f"u_{i}", lb=self.instance.demand_of_customer[i], ub=self.instance.vehicle_capacity)


        # Step 2: Add constraints to the model - in our graph, a node is a customer and the edges are the routes taken by the vehicles between customers

        # A vehicle must leave each node the same number of times it is entered
        for v in range(self.instance.num_vehicles):
            for j in range(self.instance.num_customers):
                self.model.add_constraint(
                    self.model.sum(self.edges_for_each_vehicle[v, i, j] for i in range(self.instance.num_customers) if i != j) ==
                    self.model.sum(self.edges_for_each_vehicle[v, j, k] for k in range(self.instance.num_customers) if k != j)
                )

        # Must ensure each node is visited exactly once except for the depot (range starts from 1)
        for i in range(1, self.instance.num_customers):
            self.model.add_constraint(
                self.model.sum(self.edges_for_each_vehicle[v, i, j] for j in range(self.instance.num_customers) for v in range(self.instance.num_vehicles) if i != j) == 1.0
            )

        # Must ensure every vehicle leaves + returns to the depot or does nothing
        for v in range(self.instance.num_vehicles):
            self.model.add_constraint(
                self.model.sum(self.edges_for_each_vehicle[v, 0, j] for j in range(1, self.instance.num_customers)) <= 1.0
            )

            self.model.add_constraint(
                self.model.sum(self.edges_for_each_vehicle[v, i, 0] for i in range(1, self.instance.num_customers)) <= 1.0
            )

        # Must ensure that the capacity constraint is satisfied
        for v in range(self.instance.num_vehicles):
            self.model.add_constraint(
                self.model.sum(self.edges_for_each_vehicle[v, i, j] * self.instance.demand_of_customer[j] 
                               for i in range(self.instance.num_customers) 
                               for j in range(self.instance.num_customers) 
                               if i != j) 
                <= self.instance.vehicle_capacity
            )

        # MTZ Constraints
        for v in range(self.instance.num_vehicles):
            for i in range(1, self.instance.num_customers):
                for j in range(1, self.instance.num_customers):
                    if i != j:
                        self.model.add_constraint(
                            self.u[j] - self.u[i] >= self.instance.demand_of_customer[j] - self.instance.vehicle_capacity * (1 - self.edges_for_each_vehicle[v, i, j])
                        )
        

        # Step 3: Objective Function
        def euclidean_distance(i, j):
            return np.sqrt(
                (self.instance.x_coord_of_customer[i] - self.instance.x_coord_of_customer[j]) ** 2 +
                (self.instance.y_coord_of_customer[i] - self.instance.y_coord_of_customer[j]) ** 2
            )
        
        self.model.minimize(
            self.model.sum(
                self.edges_for_each_vehicle[v, i, j] * euclidean_distance(i, j)
                for v in range(self.instance.num_vehicles)
                for i in range(self.instance.num_customers)
                for j in range(self.instance.num_customers)
                if i != j
            )
        )

    def solve(self):

        solution = self.model.solve()

        if solution:
            # self.model.print_information()
            return solution
        else:
            print("No solution found.")
            return None
        

class HopefulDream:
    def __init__(self, instance: VRPInstance):
        self.instance = instance
        self.solver = IPSolver(instance)
        pass

    def dream(self):
        min_val = float('inf')

        for i in range(1000):
            hopeful_number = np.random.randint(0, 9999999)
            self.solver.model.parameters.randomseed = hopeful_number
            self.solver.model.dettimelimit = 15
            solution = self.solver.solve()
            if solution:
                min_val = min(min_val, solution.objective_value)

            print(f"Iteration {i}: Objective Value = {solution.objective_value if solution else 'No Solution'}, Min Value = {min_val}")

        print("\n")

        
        return min_val
            

        


