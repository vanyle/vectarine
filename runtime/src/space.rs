use crate::space::transform::Transform2;

pub mod dbvh;
pub mod transform;

// TODO: generalize everything to 3D

#[derive(Clone)]
struct Entity2 {
    id: usize,
    parent_id: Option<usize>,
    transform: Transform2,
    children: Vec<usize>,
}

impl Entity2 {
    fn new(id: usize, transform: Transform2, parent_id: Option<usize>) -> Self {
        Entity2 {
            id,
            parent_id,
            transform,
            children: Vec::new(),
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

    pub fn get_children(&self) -> Vec<usize> {
        self.children.clone()
    }

    fn set_parent_id(&mut self, parent_id: Option<usize>) {
        self.parent_id = parent_id
    }

    fn set_transform(&mut self, transform: Transform2) {
        self.transform = transform
    }

    fn add_child(&mut self, child_id: usize) {
        self.children.push(child_id);
    }

    fn remove_child(&mut self, child_id: usize) {
        self.children.retain(|id| *id != child_id);
    }
}

struct Space2 {
    entities: Vec<Option<Entity2>>,
    free_spaces: Vec<usize>,
}

impl Space2 {
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
    ) -> Option<usize> {
        if self.free_spaces.len() == 0 {
            self.free_spaces.push(self.entities.len());
            self.entities.push(None)
        }
        let entity_id = self.free_spaces.pop()?;
        self.entities[entity_id] = Some(Entity2::new(entity_id, transform, parent_id));
        Some(entity_id)
    }

    pub fn delete_entity(&mut self, entity_id: usize) -> bool {
        if self.has_entity(Some(entity_id)) {
            if entity_id == self.entities.len() - 1 {
                self.entities.pop();
            } else {
                self.free_spaces.push(entity_id);
                self.entities[entity_id] = None;
            }
            true
        } else {
            false
        }
    }

    pub fn get_entity_local_transform(&mut self, entity_id: usize) -> Option<Transform2> {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut()?;
            Some(entity.transform)
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

    pub fn get_entity_absolute_transform(&mut self, entity_id: usize) -> Option<Transform2> {
        if self.has_entity(Some(entity_id)) {
            let parent_id;
            let entity_transform;
            {
                let entity = self.entities[entity_id].as_mut()?;
                parent_id = entity.get_parent_id();
                entity_transform = entity.transform.clone();
            }
            if self.has_entity(parent_id) {
                Some(self.get_entity_absolute_transform(parent_id?)? + entity_transform)
            } else {
                Some(entity_transform)
            }
        } else {
            None
        }
    }

    // Returns the new local transform
    pub fn set_entity_absolute_transform(
        &mut self,
        entity_id: usize,
        transform: Transform2,
    ) -> Option<Transform2> {
        let absolute_transform = self.get_entity_absolute_transform(entity_id);
        if absolute_transform.is_some() {
            let local_transform = self.entities[entity_id].as_mut().unwrap().transform.clone();
            // Operations on transforms are NOT commutative
            let new_transform = local_transform - absolute_transform.unwrap() + transform;
            self.set_entity_local_transform(entity_id, new_transform)
        } else {
            None
        }
    }

    pub fn get_entity_parent_id(&mut self, entity_id: usize) -> Option<usize> {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut()?;
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
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut().unwrap();
            entity.set_parent_id(None);
            true
        } else {
            false
        }
    }

    pub fn add_entity_child(&mut self, entity_id: usize, child_id: usize) -> bool {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut().unwrap();
            entity.add_child(child_id);
            true
        } else {
            false
        }
    }

    pub fn remove_entity_child(&mut self, entity_id: usize, child_id: usize) -> bool {
        if self.has_entity(Some(entity_id)) {
            let entity = self.entities[entity_id].as_mut().unwrap();
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
