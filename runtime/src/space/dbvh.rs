use crate::math::Vect;

// TODO: generalize with area-based/volume-based cost
#[derive(PartialEq, Clone, Copy)]
struct DbvhBounds<const N: usize> {
    position: Vect<N>,
    volume: Vect<N>,
}

#[derive(Clone, Copy)]
struct DbvhNode<const N: usize> {
    // TODO: store a reference to the entity linked with the node
    entity: Option<usize>,
    index: Option<usize>,
    parent_index: Option<usize>,
    child1: Option<usize>,
    child2: Option<usize>,
    is_leaf: bool,
    bounds: DbvhBounds<N>,
}

pub struct DbvhTree<const N: usize> {
    root_index: Option<usize>,
    nodes: Vec<DbvhNode<N>>,
    free_spaces: Vec<usize>,
}

impl<const N: usize> DbvhBounds<N> {
    pub fn union(&self, other: Self) -> Self {
        let min_corner = Vect::min(
            self.position - self.volume.scale(0.5),
            other.position - other.volume.scale(0.5),
        );
        let max_corner = Vect::max(
            self.position + self.volume.scale(0.5),
            other.position + other.volume.scale(0.5),
        );
        return Self {
            position: (min_corner + max_corner).scale(0.5),
            volume: max_corner - min_corner,
        };
    }

    pub fn intersection(&self, other: Self) -> Self {
        let self_min_corner = self.position - self.volume.scale(0.5);
        let self_max_corner = self.position + self.volume.scale(0.5);
        let other_min_corner = other.position - other.volume.scale(0.5);
        let other_max_corner = other.position + other.volume.scale(0.5);
        let position = Vect::max(self_min_corner, other_min_corner);
        let volume = Vect::max(
            Vect::zero(),
            Vect::min(self_max_corner, other_max_corner) - position,
        );
        return Self { position, volume };
    }

    pub fn strictlyIncludes(&self, other: Self) -> bool {
        let self_min_corner = self.position - self.volume.scale(0.5);
        let self_max_corner = self.position + self.volume.scale(0.5);
        let other_min_corner = other.position - other.volume.scale(0.5);
        let other_max_corner = other.position + other.volume.scale(0.5);
        return self_min_corner < other_min_corner && self_max_corner > other_max_corner;
    }

    pub fn overlaps(&self, other: Self) -> bool {
        let self_min_corner = self.position - self.volume.scale(0.5);
        let self_max_corner = self.position + self.volume.scale(0.5);
        let other_min_corner = other.position - other.volume.scale(0.5);
        let other_max_corner = other.position + other.volume.scale(0.5);
        return Vect::min(self_max_corner, other_max_corner)
            > Vect::max(self_min_corner, other_min_corner);
    }

    pub fn volume(&self) -> f32 {
        return self.volume.hvolume();
    }

    fn cost(&self) -> f32 {
        // 	 For ray casting the best is to use harea (in 2d it's the perimeter)
        // 	 For space partitionning only, it is best to use hvolume (in 2d it's the area)
        return self.volume();
    }
}

impl<const N: usize> DbvhNode<N> {}

impl<const N: usize> DbvhTree<N> {
    // TODO: update when space/entity structure is defined to have
    // node_index, position and volume in a single object
    pub fn is_entity_up_to_date(
        &self,
        node_index: Option<usize>,
        position: Vect<N>,
        volume: Vect<N>,
    ) -> bool {
        let Some(node_index) = node_index else {
            return true;
        };
        return self.nodes[node_index]
            .bounds
            .strictlyIncludes(DbvhBounds { position, volume });
    }

    // TODO: update when space/entity structure is defined to have node position and volume
    // in a single object
    fn align_dbvh_leaf_with_entity(
        &mut self,
        node_index: Option<usize>,
        position: Vect<N>,
        volume: Vect<N>,
    ) {
        let Some(node_index) = node_index else {
            return;
        };
        self.nodes[node_index].bounds.position = position;
        // TODO: put 1.2 in a constant ?
        self.nodes[node_index].bounds.volume = volume.scale(1.2);
    }

    fn tree_cost(&mut self, index: Option<usize>) -> f32 {
        let Some(index) = index else {
            return 0.0;
        };
        let cost = self.nodes[index].bounds.cost();
        if self
            .root_index
            .is_some_and(|root_index| index == root_index)
        {
            return cost;
        }
        return cost + self.tree_cost(self.nodes[index].parent_index);
    }

    fn delta_cost(&mut self, index: Option<usize>, bounds: DbvhBounds<N>) -> f32 {
        // Compute the additional cost for the whole tree if the node
        // at `index` were to be resized to include the `bounds`
        let Some(index) = index else {
            return 0.0;
        };
        // For ray casting the best is to use area (in 2d it's the perimeter)
        // For space partitionning only, it is best to use volume (in 2d it's the area)
        let new_bounds = self.nodes[index].bounds.union(bounds);
        if new_bounds == self.nodes[index].bounds {
            return 0.0;
        }
        let curent_cost = self.nodes[index].bounds.cost();
        let new_cost = new_bounds.cost();
        let delta = new_cost - curent_cost;
        if self
            .root_index
            .is_some_and(|root_index| index == root_index)
        {
            return delta;
        }
        return delta + self.delta_cost(self.nodes[index].parent_index, new_bounds);
    }

    fn minimize_dbvh_sub_tree(&mut self, grand_parent_index: Option<usize>) {
        // from     G  |-> parent_a    (| -> a1
        //                              | -> a2)?
        //             |-> parent_b    (| -> b1
        //                              | -> b2)?
        //
        // to
        // code 0: no change
        //
        // code 1:   G  |-> parent_a    | -> parent_b
        //                              | -> a2
        //              |-> a1
        //
        // code 2:   G  |-> parent_a    | -> a1
        //                              | -> parent_b
        //              |-> a2
        //
        // code 3:   G  |-> b1
        //              |-> parent_b    | -> parent_a
        //                              | -> b2
        //
        // code 4:   G  |-> b2
        //              |-> parent_b    | -> b1
        //                              | -> parent_a
        //
        // code 5:   G  |-> parent_a'   | -> b1
        //                              | -> a2
        //              |-> parent_b'   | -> a1
        //                              | -> b2
        //
        // code 6:   G  |-> parent_a''  | -> b2
        //                              | -> a2
        //              |-> parent_b''  | -> b1
        //                              | -> a1
        //
        let Some(grand_parent_index) = grand_parent_index else {
            return;
        };
        let Some(parent_a) = self.nodes[grand_parent_index].child1 else {
            return;
        };
        let Some(parent_b) = self.nodes[grand_parent_index].child2 else {
            return;
        };
        let mut best_swap_code = 0;
        let mut best_swap_delta_cost = 0.0;
        let mut swap_delta_cost: f32;
        if !self.nodes[parent_a].is_leaf {
            let Some(a1) = self.nodes[parent_a].child1 else {
                return;
            };
            let Some(a2) = self.nodes[parent_a].child2 else {
                return;
            };
            // swap 1: swap a1 and parent_b
            swap_delta_cost = self.nodes[a2]
                .bounds
                .union(self.nodes[parent_b].bounds)
                .cost()
                - self.nodes[parent_a].bounds.cost();
            if swap_delta_cost < best_swap_delta_cost {
                best_swap_code = 1;
                best_swap_delta_cost = swap_delta_cost;
            }
            // swap 2: swap a2 and parent_b
            swap_delta_cost = self.nodes[a1]
                .bounds
                .union(self.nodes[parent_b].bounds)
                .cost()
                - self.nodes[parent_a].bounds.cost();
            if swap_delta_cost < best_swap_delta_cost {
                best_swap_code = 2;
                best_swap_delta_cost = swap_delta_cost;
            }
        }
        if !self.nodes[parent_b].is_leaf {
            let Some(b1) = self.nodes[parent_b].child1 else {
                return;
            };
            let Some(b2) = self.nodes[parent_b].child2 else {
                return;
            };
            // swap 3: swap b1 and parent_a
            swap_delta_cost = self.nodes[b2]
                .bounds
                .union(self.nodes[parent_a].bounds)
                .cost()
                - self.nodes[parent_b].bounds.cost();
            if swap_delta_cost < best_swap_delta_cost {
                best_swap_code = 3;
                best_swap_delta_cost = swap_delta_cost;
            }
            // swap 4: swap b2 and parent_a
            swap_delta_cost = self.nodes[b1]
                .bounds
                .union(self.nodes[parent_a].bounds)
                .cost()
                - self.nodes[parent_b].bounds.cost();
            if swap_delta_cost < best_swap_delta_cost {
                best_swap_code = 4;
                best_swap_delta_cost = swap_delta_cost;
            }
        }
        if !self.nodes[parent_a].is_leaf && !self.nodes[parent_b].is_leaf {
            let Some(a1) = self.nodes[parent_a].child1 else {
                return;
            };
            let Some(a2) = self.nodes[parent_a].child2 else {
                return;
            };
            let Some(b1) = self.nodes[parent_b].child1 else {
                return;
            };
            let Some(b2) = self.nodes[parent_b].child2 else {
                return;
            };
            let initial_cost =
                self.nodes[parent_a].bounds.cost() + self.nodes[parent_b].bounds.cost();
            // swap 5: swap a1 and b1 (a1-b1 and a2-b2 are symetric swaps)
            swap_delta_cost = self.nodes[b1].bounds.union(self.nodes[a2].bounds).cost()
                + self.nodes[a1].bounds.union(self.nodes[b2].bounds).cost()
                - initial_cost;
            if swap_delta_cost < best_swap_delta_cost {
                best_swap_code = 5;
                best_swap_delta_cost = swap_delta_cost;
            }
            // swap 6: swap a1 and b2 (a1-b2 and a2-b1 are symetric swaps)
            swap_delta_cost = self.nodes[b2].bounds.union(self.nodes[a2].bounds).cost()
                + self.nodes[a1].bounds.union(self.nodes[b1].bounds).cost()
                - initial_cost;
            if swap_delta_cost < best_swap_delta_cost {
                best_swap_code = 6;
            }
        }
        match best_swap_code {
            1 => {
                // swap a1 and parent_b
                let Some(a1) = self.nodes[parent_a].child1 else {
                    return;
                };
                let Some(a2) = self.nodes[parent_a].child2 else {
                    return;
                };
                self.nodes[a1].parent_index = Some(grand_parent_index);
                self.nodes[grand_parent_index].child2 = Some(a1);
                self.nodes[parent_b].parent_index = Some(parent_a);
                self.nodes[parent_a].child1 = Some(parent_b);
                self.nodes[parent_a].bounds =
                    self.nodes[parent_b].bounds.union(self.nodes[a2].bounds);
            }
            2 => {
                // swap a2 and parent_b
                let Some(a1) = self.nodes[parent_a].child1 else {
                    return;
                };
                let Some(a2) = self.nodes[parent_a].child2 else {
                    return;
                };
                self.nodes[a2].parent_index = Some(grand_parent_index);
                self.nodes[grand_parent_index].child2 = Some(a2);
                self.nodes[parent_b].parent_index = Some(parent_a);
                self.nodes[parent_a].child2 = Some(parent_b);
                self.nodes[parent_a].bounds =
                    self.nodes[a1].bounds.union(self.nodes[parent_b].bounds);
            }
            3 => {
                // swap b1 and parent_a
                let Some(b1) = self.nodes[parent_b].child1 else {
                    return;
                };
                let Some(b2) = self.nodes[parent_b].child2 else {
                    return;
                };
                self.nodes[b1].parent_index = Some(grand_parent_index);
                self.nodes[grand_parent_index].child1 = Some(b1);
                self.nodes[parent_a].parent_index = Some(parent_b);
                self.nodes[parent_b].child1 = Some(parent_a);
                self.nodes[parent_b].bounds =
                    self.nodes[parent_a].bounds.union(self.nodes[b2].bounds);
            }
            4 => {
                // swap b2 and parent_a
                let Some(b1) = self.nodes[parent_b].child1 else {
                    return;
                };
                let Some(b2) = self.nodes[parent_b].child2 else {
                    return;
                };
                self.nodes[b2].parent_index = Some(grand_parent_index);
                self.nodes[grand_parent_index].child1 = Some(b2);
                self.nodes[parent_a].parent_index = Some(parent_b);
                self.nodes[parent_b].child2 = Some(parent_a);
                self.nodes[parent_b].bounds =
                    self.nodes[b1].bounds.union(self.nodes[parent_a].bounds);
            }
            5 => {
                // swap a1 and b1 (a1-b1 and a2-b2 are symetric swaps)
                let Some(a1) = self.nodes[parent_a].child1 else {
                    return;
                };
                let Some(a2) = self.nodes[parent_a].child2 else {
                    return;
                };
                let Some(b1) = self.nodes[parent_b].child1 else {
                    return;
                };
                let Some(b2) = self.nodes[parent_b].child2 else {
                    return;
                };
                self.nodes[a1].parent_index = Some(parent_b);
                self.nodes[parent_b].child1 = Some(a1);
                self.nodes[b1].parent_index = Some(parent_a);
                self.nodes[parent_a].child1 = Some(b1);
                self.nodes[parent_a].bounds = self.nodes[b1].bounds.union(self.nodes[a2].bounds);
                self.nodes[parent_b].bounds = self.nodes[a1].bounds.union(self.nodes[b2].bounds);
            }
            6 => {
                // swap a1 and b2 (a1-b2 and a2-b1 are symetric swaps)
                let Some(a1) = self.nodes[parent_a].child1 else {
                    return;
                };
                let Some(a2) = self.nodes[parent_a].child2 else {
                    return;
                };
                let Some(b1) = self.nodes[parent_b].child1 else {
                    return;
                };
                let Some(b2) = self.nodes[parent_b].child2 else {
                    return;
                };
                self.nodes[a1].parent_index = Some(parent_b);
                self.nodes[parent_b].child2 = Some(a1);
                self.nodes[b2].parent_index = Some(parent_a);
                self.nodes[parent_a].child1 = Some(b2);
                self.nodes[parent_a].bounds = self.nodes[b2].bounds.union(self.nodes[a2].bounds);
                self.nodes[parent_b].bounds = self.nodes[a1].bounds.union(self.nodes[b1].bounds);
            }
            _ => {}
        }
    }

    pub fn refit_dbvh_tree(&mut self, start_index: Option<usize>) {
        let Some(start_index) = start_index else {
            return;
        };
        // Refit the branch from start_index to the root in O(ln(n))
        let mut parent_index_option = self.nodes[start_index].parent_index;
        loop {
            let Some(parent_index) = parent_index_option else {
                return;
            };
            let parent = self.nodes[parent_index];
            let Some(child1) = parent.child1 else {
                return;
            };
            let Some(child2) = parent.child2 else {
                return;
            };
            self.nodes[parent_index].bounds =
                self.nodes[child1].bounds.union(self.nodes[child2].bounds);
            self.minimize_dbvh_sub_tree(Some(parent_index));
            parent_index_option = parent.parent_index;
        }
    }

    pub fn allocate_node(&mut self) {
        let node_index = self.nodes.len();
        let new_node = DbvhNode {
            entity: None,
            index: Some(node_index),
            parent_index: None,
            child1: None,
            child2: None,
            is_leaf: false,
            bounds: DbvhBounds {
                position: Vect::zero(),
                volume: Vect::zero(),
            },
        };
        self.nodes.push(new_node);
        self.free_spaces.push(node_index);
    }

    pub fn free_node(&mut self, node_index: Option<usize>) {
        let Some(node_index) = node_index else {
            return;
        };
        if node_index == self.nodes.len() - 1 {
            self.nodes.pop();
        } else {
            self.free_spaces.push(node_index)
        }
    }

    // TODO: update when space/entity structure is defined
    // entity_id might not be relevant anymore
    pub fn instantiate_node(
        &mut self,
        parent_index: Option<usize>,
        entity_id: Option<usize>,
    ) -> Option<usize> {
        if self.free_spaces.is_empty() {
            self.allocate_node();
        }
        let Some(node_index) = self.free_spaces.pop() else {
            return None;
        };
        self.nodes[node_index].parent_index = parent_index;
        self.nodes[node_index].child1 = entity_id;
        self.nodes[node_index].is_leaf = entity_id.is_some();
        return Some(node_index);
    }

    pub fn insert_dbvh_leaf(
        &mut self,
        node_index: Option<usize>,
        sibling_index: Option<usize>,
    ) -> Option<usize> {
        // Insert a node next to a given sibling
        // returns the index of the common parent
        // from     G |-> sibling
        //
        // to       G |-> P |-> node
        //                  |-> sibling
        //
        let Some(node_index) = node_index else {
            return None;
        };
        let Some(sibling_index) = sibling_index else {
            return None;
        };
        let grand_parent_index = self.nodes[sibling_index].parent_index;
        let parent_index = self.instantiate_node(grand_parent_index, None);
        let Some(parent_index) = parent_index else {
            return None;
        };
        self.nodes[sibling_index].parent_index = Some(parent_index);
        self.nodes[node_index].parent_index = Some(parent_index);
        self.nodes[parent_index].child1 = Some(sibling_index);
        self.nodes[parent_index].child2 = Some(node_index);
        // Edge case: the node is inserted directly at the root of the tree
        if self
            .root_index
            .is_some_and(|root_index| sibling_index == root_index)
        {
            self.root_index = Some(parent_index);
            return self.root_index;
        }
        let Some(grand_parent_index) = grand_parent_index else {
            return None;
        };
        if self.nodes[grand_parent_index]
            .child1
            .is_some_and(|child1| child1 == sibling_index)
        {
            self.nodes[grand_parent_index].child1 = Some(parent_index);
        } else {
            self.nodes[grand_parent_index].child2 = Some(parent_index);
        }
        return Some(parent_index);
    }

    pub fn delete_dbvh_leaf(&mut self, node_index: Option<usize>) -> bool {
        // Delete a node and its parent and attach directly its sibling to its grandparent
        // from    G |-> P |-> node (node_index)
        //                 |-> sibling
        //
        // to      G |-> sibling
        let Some(node_index) = node_index else {
            return false;
        };
        // Edge case: the leaf is also the root node
        if self
            .root_index
            .is_some_and(|root_index| node_index == root_index)
        {
            self.nodes = Vec::new();
            self.free_spaces = Vec::new();
            self.root_index = None;
            return true;
        }
        let parent_index = self.nodes[node_index].parent_index;
        let Some(parent_index) = parent_index else {
            return false;
        };
        let mut sibling_index: Option<usize> = None;
        if self.nodes[parent_index]
            .child1
            .is_some_and(|child1| child1 == node_index)
        {
            sibling_index = self.nodes[parent_index].child2;
        }
        if self.nodes[parent_index]
            .child2
            .is_some_and(|child2| child2 == node_index)
        {
            sibling_index = self.nodes[parent_index].child1;
        }
        let Some(sibling_index) = sibling_index else {
            return false;
        };
        // 1) Delete node
        self.free_node(Some(node_index));
        // Edge case: parent of the leaf is the root node
        if self
            .root_index
            .is_some_and(|root_index| parent_index == root_index)
        {
            self.root_index = Some(sibling_index);
            self.nodes[sibling_index].parent_index = None;
            self.free_node(Some(parent_index));
            return true;
        }
        // 2) Add sibling to grandparent
        let grand_parent_index = self.nodes[parent_index].parent_index.unwrap();
        self.nodes[sibling_index].parent_index = Some(grand_parent_index);
        if self.nodes[grand_parent_index]
            .child1
            .is_some_and(|child1| child1 == parent_index)
        {
            self.nodes[grand_parent_index].child1 = Some(sibling_index);
        } else {
            self.nodes[grand_parent_index].child2 = Some(sibling_index);
        }
        // 3) Delete parent
        self.free_node(Some(parent_index));
        // 4) Refit tree
        self.refit_dbvh_tree(Some(grand_parent_index));
        return true;
    }

    // TODO: update when space/entity structure is defined to have
    // entity_id, position and volume in a single object
    pub fn create_dbvh_leaf(
        &mut self,
        entity_id: Option<usize>,
        position: Vect<N>,
        volume: Vect<N>,
    ) -> Option<usize> {
        let node_index = self.instantiate_node(None, entity_id);
        self.align_dbvh_leaf_with_entity(node_index, position, volume);
        // Edge case: first leaf of the tree
        if self.root_index.is_none() {
            self.root_index = node_index;
            return self.root_index;
        }
        let Some(node_index) = node_index else {
            return None;
        };
        // 1) Search for best position based on area cost
        let new_node_cost = self.nodes[node_index].bounds.cost();
        let mut best_cost: f32 = f32::MAX;
        let mut sibling_index = self.root_index;
        let mut i = 0;
        let mut candidates: Vec<Option<usize>> = Vec::new();
        candidates.push(sibling_index);
        while i < candidates.len() {
            let local_cost = self.tree_cost(candidates[i]);
            // TODO: optimisation possible: store delta costs to avoid recomputing them everytime
            // could also accelerate the refitting (phase 3) by stopping earlier
            let delta = self.delta_cost(candidates[i], self.nodes[node_index].bounds);
            // Adding the node here would cause an increase of cost cooresponding to
            // the node itself (constant cost wherever the node is placed so no need
            // to take it into account) and the parent node containing both the new
            // node and the current sibling candidate (for a total of local_cost + delta
            // because delta is precisely the additionnal cost needed to account for the
            // total cost of the sibling node and the bounds of the new node)
            let cost = local_cost + delta;
            if cost < best_cost {
                best_cost = cost;
                sibling_index = candidates[i];
            }
            // The lower bound for a child cost is new_node_cost + delta
            // because it will necessarily increased its parent cost by delta
            // and the created child node will include the new node s its cost
            // will be of at least new_node_cost
            if candidates[i].is_some_and(|candidate| !self.nodes[candidate].is_leaf)
                && new_node_cost + delta < best_cost
            {
                candidates.push(self.nodes[candidates[i].unwrap()].child1);
                candidates.push(self.nodes[candidates[i].unwrap()].child2);
            }
            i += 1;
        }
        // 2) insert leaf at best position
        self.insert_dbvh_leaf(Some(node_index), sibling_index);

        // 3) Propagate change
        self.refit_dbvh_tree(Some(node_index));
        return Some(node_index);
    }
}
