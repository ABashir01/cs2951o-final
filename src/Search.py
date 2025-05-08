from simanneal import Annealer

class VRPAnnealer(Annealer):
    def __init__(self, state, instance: VRPInstance):
        self.instance = instance
        super(VRPAnnealer, self).__init__(state)

    def move(self):
        pass

    def energy(self):
        pass

def search(instance: VRPInstance, init):
    vrp = VRPAnnealer(init, instance)

    # automatically search for a schedule
    sched = vrp.auto(minutes=0.5) 
    vrp.set_schedule(sched)

    sol, dist = vrp.anneal()
    return sol, dist
