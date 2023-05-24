use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use ambient_api::prelude::*;
use parry2d::{bounding_volume::Aabb, math::Point, partitioning::*};
use slab::Slab;

use crate::components::map_position;

struct PartitioningData {
    qbvh: Qbvh<usize>,
    leaves_to_entities: Slab<EntityId>,
    entities_to_leaves: HashMap<EntityId, usize>,
    workspace: QbvhUpdateWorkspace,
}

impl PartitioningData {
    fn split_borrow(&mut self) -> (&mut Qbvh<usize>, &mut QbvhUpdateWorkspace) {
        (&mut self.qbvh, &mut self.workspace)
    }
}

pub fn init_qbvh(
    searchable_component: Component<()>,
    search_radius_component: Component<f32>,
    result_component: Component<EntityId>,
) {
    let data = PartitioningData {
        qbvh: Qbvh::new(),
        leaves_to_entities: Slab::new(),
        entities_to_leaves: HashMap::new(),
        workspace: QbvhUpdateWorkspace::default(),
    };

    let data = Arc::new(RwLock::new(data));

    change_query((map_position(), searchable_component))
        .track_change((map_position(), searchable_component))
        .bind({
            let data = data.clone();
            move |entities| {
                let mut data = data.write().unwrap();
                let mut pos_cache = HashMap::with_capacity(entities.len());
                for (e, (pos, _searchable)) in entities {
                    let leaf = if let Some(leaf) = data.entities_to_leaves.get(&e) {
                        *leaf
                    } else {
                        let leaf = data.leaves_to_entities.insert(e);
                        data.entities_to_leaves.insert(e, leaf);
                        leaf
                    };

                    pos_cache.insert(leaf, pos);
                    data.qbvh.pre_update_or_insert(leaf);
                }

                let (qbvh, workspace) = data.split_borrow();
                qbvh.refit(0.01, workspace, |leaf| {
                    if let Some(pos) = pos_cache.get(&leaf) {
                        let pos = Point::new(pos.x, pos.y);
                        Aabb::new(pos, pos)
                    } else {
                        Aabb::new_invalid()
                    }
                });
                qbvh.rebalance(0.01, workspace);
            }
        });

    despawn_query((map_position(), searchable_component)).bind({
        let data = data.clone();
        move |entities| {
            let mut data = data.write().unwrap();
            for (e, (_pos, _searchable)) in entities {
                if let Some(leaf) = data.entities_to_leaves.remove(&e) {
                    data.qbvh.remove(leaf);
                    data.leaves_to_entities.remove(leaf);
                }
            }
        }
    });

    query((map_position(), search_radius_component))
        .excludes(result_component)
        .each_frame({
            let data = data.clone();
            move |entities| {
                let data = data.read().unwrap();
                let mut query_results = Vec::new();
                for (e, (search_pos, search_radius)) in entities {
                    let mins = search_pos - search_radius;
                    let maxs = search_pos + search_radius;
                    let mins = Point::new(mins.x, mins.y);
                    let maxs = Point::new(maxs.x, maxs.y);
                    let search_aabb = Aabb::new(mins, maxs);
                    data.qbvh.intersect_aabb(&search_aabb, &mut query_results);

                    let mut closest_result = EntityId::null();
                    let mut closest_distance = search_radius;
                    for result_leaf in query_results.iter().copied() {
                        let result = match data.leaves_to_entities.get(result_leaf) {
                            Some(result) => *result,
                            None => continue,
                        };

                        // TODO cache this in EntityLeaf?
                        let result_pos = match entity::get_component(result, map_position()) {
                            None => continue,
                            Some(pos) => pos,
                        };

                        let distance = result_pos.distance(search_pos);
                        if distance < closest_distance {
                            closest_result = result;
                            closest_distance = distance;
                        }
                    }

                    entity::add_component(e, result_component, closest_result);

                    query_results.clear();
                }
            }
        });
}
