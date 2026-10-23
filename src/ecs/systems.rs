use crate::core::context::Context;
use crate::ecs::components::Transform;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use glam::Mat4;
use slotmap::SecondaryMap;

/// Traverses transform hierarchy from root entities down child branches, updating cached `global_transform` matrices.
///
/// Executes without per-frame heap allocations by utilizing recursive traversal on the stack.
pub fn update_hierarchy_system(world: &mut World) {
    let transforms = &world.transform;
    let parents = &world.parent;
    let children = &world.children;
    let global_transforms = &mut world.global_transform;

    for (entity, transform) in transforms {
        if parents.get(entity).is_none() {
            let global = transform.matrix();
            global_transforms.insert(entity, global);

            if let Some(c_list) = children.get(entity) {
                for &child in c_list {
                    propagate_child(child, global, transforms, global_transforms, children);
                }
            }
        }
    }
}

fn propagate_child(
    child: EntityId,
    parent_global: Mat4,
    transforms: &SecondaryMap<EntityId, Transform>,
    global_transforms: &mut SecondaryMap<EntityId, Mat4>,
    children: &SecondaryMap<EntityId, Vec<EntityId>>,
) {
    let local = transforms
        .get(child)
        .map(|t| t.matrix())
        .unwrap_or(Mat4::IDENTITY);
    let global = parent_global * local;
    global_transforms.insert(child, global);

    if let Some(c_list) = children.get(child) {
        for &grandchild in c_list {
            propagate_child(grandchild, global, transforms, global_transforms, children);
        }
    }
}

/// Executes all procedural [`super::components::NodeBehavior`] scripts attached to entities.
pub fn update_behaviors_system(world: &mut World, ctx: &mut Context, dt: f32) {
    let entities: Vec<EntityId> = world.behaviors.keys().collect();
    for entity in entities {
        if let Some(mut behavior) = world.behaviors.remove(entity) {
            behavior.on_process(entity, world, ctx, dt);
            if world.is_alive(entity) {
                world.behaviors.insert(entity, behavior);
            }
        }
    }
}
