use crate::gui::follow::FollowEntity;
use crate::gui::roadeditor::IntersectionComponent;
use egregoria::map_dynamic::Itinerary;
use egregoria::pedestrians::{Location, Pedestrian};
use egregoria::physics::{Collider, Kinematics};
use egregoria::rendering::assets::AssetRender;
use egregoria::rendering::meshrender_component::MeshRender;
use egregoria::vehicles::Vehicle;
use egregoria::Egregoria;
use geom::Transform;
use imgui::im_str;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use legion::storage::Component;
use legion::Entity;

pub struct InspectRenderer {
    pub entity: Entity,
}

/// Avoids Cloning by mutably aliasing the component inside the world
/// Unsound if the inspector also try to get the component using the world borrow
fn modify<T: Component>(
    goria: &mut Egregoria,
    entity: Entity,
    f: impl FnOnce(&mut T) -> bool,
) -> Option<bool> {
    let c = goria.comp_mut::<T>(entity)?;
    Some(f(c))
}

impl InspectRenderer {
    fn inspect_component<T: Component + InspectRenderDefault<T>>(
        &self,
        world: &mut Egregoria,
        ui: &Ui,
    ) -> bool {
        modify(world, self.entity, |x| -> bool {
            <T as InspectRenderDefault<T>>::render_mut(
                &mut [x],
                std::any::type_name::<T>().split("::").last().unwrap_or(""),
                ui,
                &InspectArgsDefault::default(),
            )
        })
        .unwrap_or(false)
    }

    pub fn render(&self, goria: &mut Egregoria, ui: &Ui) -> bool {
        let mut dirty = false;

        dirty |= self.inspect_component::<Transform>(goria, ui);
        dirty |= self.inspect_component::<Vehicle>(goria, ui);
        dirty |= self.inspect_component::<Pedestrian>(goria, ui);
        dirty |= self.inspect_component::<Location>(goria, ui);
        dirty |= self.inspect_component::<AssetRender>(goria, ui);
        dirty |= self.inspect_component::<MeshRender>(goria, ui);
        dirty |= self.inspect_component::<Kinematics>(goria, ui);
        dirty |= self.inspect_component::<Collider>(goria, ui);
        dirty |= self.inspect_component::<IntersectionComponent>(goria, ui);
        dirty |= self.inspect_component::<Itinerary>(goria, ui);

        {
            let follow = &mut goria.write::<FollowEntity>().0;
            if follow.is_none() {
                if ui.small_button(im_str!("Follow")) {
                    follow.replace(self.entity);
                }
            } else if ui.small_button(im_str!("Unfollow")) {
                follow.take();
            }
        }

        if dirty {
            ui.text("dirty");
        }

        dirty
    }
}
