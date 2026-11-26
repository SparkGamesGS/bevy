use accesskit::Role;
use bevy_a11y::AccessibilityNode;
use bevy_app::{App, Plugin};
use bevy_ecs::query::Has;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    observer::On,
    query::With,
    reflect::ReflectComponent,
    system::{Commands, Query},
};
use bevy_input::keyboard::{KeyCode, KeyboardInput};
use bevy_input::ButtonState;
use bevy_input_focus::FocusedInput;
use bevy_picking::events::{
    PointerCancel, PointerClick, PointerDragEnd, PointerPress, PointerRelease,
};
use bevy_reflect::Reflect;
use bevy_ui::{InteractionDisabled, Pressed};

use crate::Activate;

/// Headless button widget. This widget maintains a "pressed" state, which is used to
/// indicate whether the button is currently being pressed by the user. It emits an [`Activate`]
/// event when the button is un-pressed.
#[derive(Component, Default, Debug, Clone)]
#[require(AccessibilityNode(accesskit::Node::new(Role::Button)))]
#[derive(Reflect)]
#[reflect(Component)]
pub struct Button;

/// Optional marker component that indicates we want the button to activate on the pointer down
/// event, this is used for menu buttons.
#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct ActivateOnPress;

fn button_on_key_event(
    mut event: On<FocusedInput<KeyboardInput>>,
    q_state: Query<Has<InteractionDisabled>, With<Button>>,
    mut commands: Commands,
) {
    if let Ok(disabled) = q_state.get(event.focused_entity)
        && !disabled
    {
        let input_event = &event.input;
        if !input_event.repeat
            && input_event.state == ButtonState::Pressed
            && (input_event.key_code == KeyCode::Enter || input_event.key_code == KeyCode::Space)
        {
            event.propagate(false);
            commands.trigger(Activate {
                entity: event.focused_entity,
            });
        }
    }
}

fn button_on_pointer_click(
    mut click: On<PointerClick>,
    mut q_state: Query<
        (Has<Pressed>, Has<InteractionDisabled>, Has<ActivateOnPress>),
        With<Button>,
    >,
    mut commands: Commands,
) {
    if let Ok((pressed, disabled, activate_on_press)) = q_state.get_mut(click.entity) {
        click.propagate(false);
        if pressed && !disabled && !activate_on_press {
            commands.trigger(Activate {
                entity: click.entity,
            });
        }
    }
}

fn button_on_pointer_down(
    mut press: On<PointerPress>,
    mut q_state: Query<
        (
            Entity,
            Has<InteractionDisabled>,
            Has<Pressed>,
            Has<ActivateOnPress>,
        ),
        With<Button>,
    >,
    mut commands: Commands,
) {
    if let Ok((button, disabled, pressed, activate_on_press)) = q_state.get_mut(press.entity) {
        press.propagate(false);
        if !disabled && !pressed {
            commands.entity(button).insert(Pressed);
            if activate_on_press {
                commands.trigger(Activate { entity: button });
            }
        }
    }
}

fn button_on_pointer_up(
    mut release: On<PointerRelease>,
    mut q_state: Query<(Entity, Has<InteractionDisabled>, Has<Pressed>), With<Button>>,
    mut commands: Commands,
) {
    if let Ok((button, disabled, pressed)) = q_state.get_mut(release.entity) {
        release.propagate(false);
        if !disabled && pressed {
            commands.entity(button).remove::<Pressed>();
        }
    }
}

fn button_on_pointer_drag_end(
    mut drag_end: On<PointerDragEnd>,
    mut q_state: Query<(Entity, Has<InteractionDisabled>, Has<Pressed>), With<Button>>,
    mut commands: Commands,
) {
    if let Ok((button, disabled, pressed)) = q_state.get_mut(drag_end.entity) {
        drag_end.propagate(false);
        if !disabled && pressed {
            commands.entity(button).remove::<Pressed>();
        }
    }
}

fn button_on_pointer_cancel(
    mut cancel: On<PointerCancel>,
    mut q_state: Query<(Entity, Has<InteractionDisabled>, Has<Pressed>), With<Button>>,
    mut commands: Commands,
) {
    if let Ok((button, disabled, pressed)) = q_state.get_mut(cancel.entity) {
        cancel.propagate(false);
        if !disabled && pressed {
            commands.entity(button).remove::<Pressed>();
        }
    }
}

/// Plugin that adds the observers for the [`Button`] widget.
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(button_on_key_event)
            .add_observer(button_on_pointer_down)
            .add_observer(button_on_pointer_up)
            .add_observer(button_on_pointer_click)
            .add_observer(button_on_pointer_drag_end)
            .add_observer(button_on_pointer_cancel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::PreUpdate;
    use bevy_ecs::{
        message::MessageWriter, resource::Resource, schedule::IntoScheduleConfigs, system::ResMut,
        system::Single,
    };
    use bevy_input::touch::{TouchInput, TouchPhase};
    use bevy_math::Vec2;
    use bevy_picking::{
        backend::{HitData, PointerHits},
        input::PointerInputPlugin,
        pointer::{PointerId, PointerLocation},
        InteractionPlugin, PickingPlugin, PickingSystems,
    };
    use bevy_window::{PrimaryWindow, Window, WindowEvent};

    #[derive(Resource, Default)]
    struct Activations(usize);

    /// A stand-in picking backend: every located pointer is over the button.
    fn hit_button(
        pointers: Query<(&PointerId, &PointerLocation)>,
        button: Single<Entity, With<Button>>,
        mut hits: MessageWriter<PointerHits>,
    ) {
        for (id, location) in &pointers {
            if location.location().is_some() {
                let hit = HitData::new(*button, 0.0, None, None);
                hits.write(PointerHits::new(*id, vec![(*button, hit)], 0.0));
            }
        }
    }

    /// A touch that starts and ends within one frame (a quick tap on a slow
    /// device, or `adb shell input tap`) activates the button and leaves it
    /// unpressed.
    #[test]
    fn touch_starting_and_ending_in_one_frame_activates() {
        let mut app = App::new();
        app.add_message::<WindowEvent>()
            .add_message::<TouchInput>()
            .add_plugins((
                PickingPlugin,
                InteractionPlugin,
                PointerInputPlugin,
                ButtonPlugin,
            ))
            .init_resource::<Activations>()
            .add_systems(PreUpdate, hit_button.in_set(PickingSystems::Backend));
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let button = app
            .world_mut()
            .spawn(Button)
            .observe(|_: On<Activate>, mut n: ResMut<Activations>| n.0 += 1)
            .id();
        app.update();

        for phase in [TouchPhase::Started, TouchPhase::Ended] {
            let touch = TouchInput {
                phase,
                position: Vec2::new(10.0, 10.0),
                window,
                force: None,
                id: 1,
            };
            app.world_mut().write_message(touch);
            app.world_mut()
                .write_message(WindowEvent::TouchInput(touch));
        }
        app.update();
        app.update();

        assert_eq!(app.world().resource::<Activations>().0, 1);
        assert!(!app.world().entity(button).contains::<Pressed>());
    }
}
