use crate::{
    dbvh::{self, DbvhTree},
    space2::{
        collider2::{Collider2, Collision2, Collision2Details},
        transform2::Transform2,
    },
};

pub mod collider2;
pub mod transform2;

use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Entity2 {
    id: usize,
    parent_id: Option<usize>,
    transform: Transform2,
    children: HashSet<usize>,
    enable_collision: bool,
    collider2: Option<Collider2>,
}

impl Entity2 {
    fn new(
        id: usize,
        transform: Transform2,
        parent_id: Option<usize>,
        enable_collision: bool,
        collider2: Option<Collider2>,
    ) -> Self {
        Entity2 {
            id,
            parent_id,
            transform,
            children: HashSet::new(),
            enable_collision,
            collider2,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_parent_id(&self) -> Option<usize> {
        self.parent_id
    }

    pub fn get_transform(&self) -> Transform2 {
        self.transform
    }

    pub fn get_children(&self) -> HashSet<usize> {
        self.children.clone()
    }

    fn set_parent_id(&mut self, parent_id: Option<usize>) {
        self.parent_id = parent_id
    }

    fn set_transform(&mut self, transform: Transform2) {
        self.transform = transform
    }

    fn add_child(&mut self, child_id: usize) {
        self.children.insert(child_id);
    }

    fn remove_child(&mut self, child_id: usize) {
        self.children.remove(&child_id);
    }
}

struct Entity2Manager {
    entities: Vec<Option<Entity2>>,
    free_spaces: Vec<usize>,
}

impl Entity2Manager {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_spaces: Vec::new(),
        }
    }

    pub fn has_entity(&self, entity_id: Option<usize>) -> bool {
        if let Some(entity_id) = entity_id {
            entity_id < self.entities.len() && self.entities[entity_id].is_some()
        } else {
            false
        }
    }

    pub fn get_entity_by_id(&self, entity_id: usize) -> Option<Entity2> {
        if self.has_entity(Some(entity_id)) {
            self.entities[entity_id].clone()
        } else {
            None
        }
    }

    pub fn get_entity_children(&self, entity: Entity2) -> Vec<Entity2> {
        let mut children = Vec::new();
        for child_id in entity.get_children() {
            if let Some(child) = self.get_entity_by_id(child_id) {
                children.push(child);
            }
        }
        children
    }

    pub fn create_entity(
        &mut self,
        transform: Transform2,
        parent_id: Option<usize>,
        enable_collision: bool,
        collider2: Option<Collider2>,
    ) -> Option<usize> {
        if self.free_spaces.is_empty() {
            self.free_spaces.push(self.entities.len());
            self.entities.push(None)
        }
        let entity_id = self.free_spaces.pop()?;
        self.entities[entity_id] = Some(Entity2::new(
            entity_id,
            transform,
            parent_id,
            enable_collision,
            collider2,
        ));
        Some(entity_id)
    }

    fn delete_entity(&mut self, entity_id: usize) -> Option<Entity2> {
        if !self.has_entity(Some(entity_id)) {
            return None;
        }
        let deleted_entity = self.entities[entity_id].clone();
        self.entities[entity_id] = None;
        if entity_id == self.entities.len() - 1 {
            self.entities.pop();
        } else {
            self.free_spaces.push(entity_id);
        }
        deleted_entity
    }

    pub fn get_entity_local_transform(&self, entity_id: usize) -> Option<Transform2> {
        if self.has_entity(Some(entity_id)) {
            let transform = (self.entities[entity_id]).as_ref()?.transform;
            Some(transform)
        } else {
            None
        }
    }

    pub fn set_entity_local_transform(
        &mut self,
        entity_id: usize,
        transform: Transform2,
    ) -> Option<Transform2> {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut()?;
            entity.set_transform(transform);
            Some(transform)
        } else {
            None
        }
    }

    pub fn get_entity_absolute_transform(&self, entity_id: usize) -> Option<Transform2> {
        let entity_local_transform = self.get_entity_local_transform(entity_id)?;
        if let Some(parent_id) = self.get_entity_parent_id(entity_id) {
            let parent_transform = self.get_entity_absolute_transform(parent_id)?;
            Some(parent_transform + entity_local_transform)
        } else {
            Some(entity_local_transform)
        }
    }

    // Returns the new local transform
    pub fn set_entity_absolute_transform(
        &mut self,
        entity_id: usize,
        transform: Transform2,
    ) -> Option<Transform2> {
        if let Some(absolute_transform) = self.get_entity_absolute_transform(entity_id) {
            let local_transform = self.get_entity_local_transform(entity_id)?;
            // Operations on transforms are NOT commutative
            let new_transform = local_transform - absolute_transform + transform;
            self.set_entity_local_transform(entity_id, new_transform)
        } else {
            None
        }
    }

    pub fn get_entity_parent_id(&self, entity_id: usize) -> Option<usize> {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_ref()?;
            entity.parent_id
        } else {
            None
        }
    }

    pub fn add_entity_parent(&mut self, entity_id: usize, parent_id: usize) -> Option<usize> {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut()?;
            entity.set_parent_id(Some(parent_id));
            Some(parent_id)
        } else {
            None
        }
    }

    pub fn remove_entity_parent(&mut self, entity_id: usize) -> bool {
        if let Some(entity) = self.entities[entity_id].as_mut() {
            entity.set_parent_id(None);
            true
        } else {
            false
        }
    }

    pub fn add_entity_child(&mut self, entity_id: usize, child_id: usize) -> bool {
        if let Some(entity) = self.entities[entity_id].as_mut() {
            entity.add_child(child_id);
            true
        } else {
            false
        }
    }

    pub fn remove_entity_child(&mut self, entity_id: usize, child_id: usize) -> bool {
        if let Some(entity) = self.entities[entity_id].as_mut() {
            entity.remove_child(child_id);
            true
        } else {
            false
        }
    }

    pub fn get_entities(&self) -> impl Iterator<Item = Option<Entity2>> + '_ {
        self.entities.iter().cloned()
    }
}

struct Collision2Manager {
    active_collisions: HashMap<usize, HashMap<usize, Collision2Details>>,
    active_collisions_reverse_link: HashMap<usize, HashSet<usize>>,
    new_collisions: Vec<Collision2Details>,
    removed_collisions: Vec<Collision2Details>,
}

impl Collision2Manager {
    pub fn new() -> Self {
        Self {
            active_collisions: HashMap::new(),
            active_collisions_reverse_link: HashMap::new(),
            new_collisions: Vec::new(),
            removed_collisions: Vec::new(),
        }
    }

    pub fn update_collision(
        &mut self,
        unordered_entity_id: usize,
        unordered_adjacent_entity_id: usize,
        collisions: Vec<Collision2>,
    ) {
        // Reorder entities, to have entity_id < adjacent_entity_id
        // because it is assumed to be true in many places
        let entity_id: usize;
        let adjacent_entity_id: usize;
        if unordered_entity_id < unordered_adjacent_entity_id {
            entity_id = unordered_entity_id;
            adjacent_entity_id = unordered_adjacent_entity_id;
        } else {
            entity_id = unordered_adjacent_entity_id;
            adjacent_entity_id = unordered_entity_id;
        }

        if let std::collections::hash_map::Entry::Vacant(entry) =
            self.active_collisions.entry(entity_id)
        {
            if collisions.is_empty() {
                return;
            }
            entry.insert(HashMap::new());
        }
        // maintain bidirectionnal link to simplify access from both entities
        if let std::collections::hash_map::Entry::Vacant(entry) = self
            .active_collisions_reverse_link
            .entry(adjacent_entity_id)
        {
            entry.insert(HashSet::new());
        }

        let entity_active_collisions: &mut HashMap<usize, Collision2Details> = self
            .active_collisions
            .get_mut(&entity_id)
            .expect("Entry should exist");
        let adjacent_entity_active_collisions_reverse_link: &mut HashSet<usize> = self
            .active_collisions_reverse_link
            .get_mut(&adjacent_entity_id)
            .expect("Entry should exist");

        // Case 1) No collision
        if collisions.is_empty() {
            let previous_entry_option = entity_active_collisions.remove_entry(&adjacent_entity_id);
            if entity_active_collisions.is_empty() {
                self.active_collisions.remove_entry(&entity_id);
            }

            // also remove bidirectionnal link
            adjacent_entity_active_collisions_reverse_link.remove(&entity_id);
            if adjacent_entity_active_collisions_reverse_link.is_empty() {
                self.active_collisions_reverse_link
                    .remove_entry(&adjacent_entity_id);
            }

            // Detect collision exit
            if let Some((previous_key, previous_value)) = previous_entry_option {
                self.removed_collisions.insert(previous_key, previous_value);
            }

            return;
        };

        // Case 2) At least one collision
        let Some(collision2_details) = entity_active_collisions.get_mut(&adjacent_entity_id) else {
            let mut new_collision2_details = Collision2Details::new();
            new_collision2_details.update_collision_details(collisions);
            self.new_collisions.push(new_collision2_details.clone());
            entity_active_collisions.insert(adjacent_entity_id, new_collision2_details);
            // create bidirectionnal link to simplify access from both entities
            adjacent_entity_active_collisions_reverse_link.insert(entity_id);
            return;
        };
        collision2_details.update_collision_details(collisions);
    }
}

struct Space2 {
    entity2_manager: Entity2Manager,
    dbvh_tree: dbvh::DbvhTree<2>,
    collision2_manager: Collision2Manager,
}

impl Space2 {
    pub fn new() -> Self {
        Self {
            entity2_manager: Entity2Manager::new(),
            dbvh_tree: dbvh::DbvhTree::new(),
            collision2_manager: Collision2Manager::new(),
        }
    }

    fn delete_entity(&mut self, entity_id: usize) {
        let Some(entity) = self.entity2_manager.get_entity_by_id(entity_id) else {
            return;
        };
        // handle bidirectional link to parent
        if let Some(parent_id) = entity.parent_id {
            let parent = self.entity2_manager.entities[parent_id]
                .as_mut()
                .expect("Parent should be defined");
            parent.remove_child(entity_id);
        }
        for child_id in entity.children {
            // avoid handling bidirectional link to children twice,
            // which would cause errors
            let child = self.entity2_manager.entities[child_id]
                .as_mut()
                .expect("Child should be defined");
            child.parent_id = None;
            // delete children recursively
            self.delete_entity(child_id);
        }
        let Some(deleted_entity) = self.entity2_manager.delete_entity(entity_id) else {
            return;
        };
        if let Some(collider2) = deleted_entity.collider2 {
            self.dbvh_tree
                .delete_dbvh_leaf(Some(collider2.get_dbvh_index()));
        }

        // remove bidirectionnal link from entity
        if let Some(reverse_links) = self
            .collision2_manager
            .active_collisions_reverse_link
            .get(&deleted_entity.id)
        {
            for adjacent_entity_id in reverse_links.iter() {
                let Some(adjacent_entity_collisions) = self
                    .collision2_manager
                    .active_collisions
                    .get_mut(adjacent_entity_id)
                else {
                    continue;
                };
                adjacent_entity_collisions.remove(&entity_id);
            }
        }
        // remove bidirectionnal link from adjacent entities
        if let Some(entity_collisions) = self
            .collision2_manager
            .active_collisions
            .get(&deleted_entity.id)
        {
            for (adjacent_entity_id, _) in entity_collisions.iter() {
                let Some(adjacent_entity_reverse_links) = self
                    .collision2_manager
                    .active_collisions_reverse_link
                    .get_mut(adjacent_entity_id)
                else {
                    continue;
                };
                adjacent_entity_reverse_links.remove(&entity_id);
            }
        }
        // finally remove all collisions
        self.collision2_manager
            .active_collisions
            .remove(&deleted_entity.id);
    }

    fn get_entities_collisions(
        entity2_manager: &Entity2Manager,
        entity_collider2: Collider2,
        entity_parent_transform: Transform2,
        adjacent_entity_id: usize,
    ) -> Option<Vec<Collision2>> {
        let adjacent_entity = entity2_manager.get_entity_by_id(adjacent_entity_id)?;
        if !adjacent_entity.enable_collision {
            return None;
        }
        let adjacent_collider2 = adjacent_entity.collider2?;
        let adjacent_entity_absolute_transform =
            entity2_manager.get_entity_absolute_transform(adjacent_entity.id)?;
        let adjacent_entity_parent_transform =
            adjacent_entity_absolute_transform - adjacent_collider2.get_transform();
        let collisions = collider2::get_colliders_collisions(
            &entity_collider2,
            entity_parent_transform,
            &adjacent_collider2,
            adjacent_entity_parent_transform,
        );
        if collisions.is_empty() {
            None
        } else {
            Some(collisions)
        }
    }

    fn compute_all_entity_collisions(
        entity2_manager: &Entity2Manager,
        collision2_manager: &mut Collision2Manager,
        dbvh_tree: &DbvhTree<2>,
        entity_option: Option<&Entity2>,
    ) -> Option<usize> {
        let entity = entity_option.as_ref()?;
        if !entity.enable_collision {
            return None;
        }
        let entity_collider2 = entity.collider2.as_ref()?;
        let node_index = entity_collider2.get_dbvh_index();
        let adjacent_nodes = dbvh_tree.get_adjacent_node_indexes(node_index);
        let entity_absolute_transform = entity2_manager.get_entity_absolute_transform(entity.id)?;
        let entity_parent_transform = entity_absolute_transform - entity_collider2.get_transform();
        let mut collision_count = 0;
        for adjacent_node_index in adjacent_nodes {
            let Some(adjacent_entity_id) = dbvh_tree
                .get_node_by_index(adjacent_node_index)
                .get_entity_id()
            else {
                continue;
            };
            if adjacent_entity_id < entity.id {
                continue;
            }
            let Some(collisions) = Self::get_entities_collisions(
                entity2_manager,
                entity_collider2.clone(),
                entity_parent_transform,
                adjacent_entity_id,
            ) else {
                collision2_manager.update_collision(entity.id, adjacent_entity_id, Vec::new());
                continue;
            };
            collision2_manager.update_collision(entity.id, adjacent_entity_id, collisions);
            collision_count += 1;
        }
        Some(collision_count)
    }

    pub fn process_collisions(&mut self) {
        self.collision2_manager.new_collisions = Vec::new();
        self.collision2_manager.removed_collisions = Vec::new();
        let mut total_collision_count = 0;
        for entity in self.entity2_manager.entities.iter() {
            let Some(collision_count) = Self::compute_all_entity_collisions(
                &self.entity2_manager,
                &mut self.collision2_manager,
                &self.dbvh_tree,
                entity.as_ref(),
            ) else {
                continue;
            };
            total_collision_count += collision_count;
        }

        for new_collision in &self.collision2_manager.new_collisions {
            // TODO: process collision enter
        }

        let mut missed_collision_exit_list: Vec<(usize, usize)> = Vec::new();
        for (entity_id, entity_collisions) in self.collision2_manager.active_collisions.iter() {
            for (adjacent_entity_id, collision_details) in entity_collisions {
                // detect missed exit
                let Some(entity_collider) =
                    (|| self.entity2_manager.get_entity_by_id(*entity_id)?.collider2)()
                else {
                    missed_collision_exit_list.push((*entity_id, *adjacent_entity_id));
                    return;
                };
                let Some(adjacent_entity_collider) = (|| {
                    self.entity2_manager
                        .get_entity_by_id(*adjacent_entity_id)?
                        .collider2
                })() else {
                    missed_collision_exit_list.push((*entity_id, *adjacent_entity_id));
                    return;
                };

                if !self.dbvh_tree.are_nodes_adjacent(
                    entity_collider.get_dbvh_index(),
                    adjacent_entity_collider.get_dbvh_index(),
                ) {
                    missed_collision_exit_list.push((*entity_id, *adjacent_entity_id));
                    return;
                }

                // TODO: process collision
            }
        }

        // add missed collision exit to removed collisions
        for (entity_id, adjacent_entity_id) in missed_collision_exit_list {
            self.collision2_manager
                .update_collision(entity_id, adjacent_entity_id, Vec::new());
        }

        for new_collision in &self.collision2_manager.removed_collisions {
            // TODO: process collision exit
        }
    }
}
